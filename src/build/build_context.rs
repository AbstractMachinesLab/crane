use super::{Artifact, BuildPlan, BuildRule};
use crate::label::Label;
use crate::toolchains::Toolchain;
use crate::workspace::Workspace;
use anyhow::Context;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use log::debug;
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BuildContext {
    artifact_root: PathBuf,
    workspace: Workspace,
    toolchain: Toolchain,
    declared_outputs: Vec<PathBuf>,
    build_plan: BuildPlan,
}

impl BuildContext {
    pub fn new(workspace: Workspace, build_plan: BuildPlan) -> BuildContext {
        let ctx = BuildContext {
            artifact_root: workspace.root().join(".crane"),
            toolchain: workspace.toolchains(),
            declared_outputs: vec![],
            workspace,
            build_plan,
        };
        fs::create_dir_all(ctx.output_path()).unwrap();
        fs::create_dir_all(ctx.cache_path()).unwrap();
        ctx
    }

    pub fn toolchain(&self) -> Toolchain {
        self.toolchain.clone()
    }

    pub fn find_node(&self, label: &Label) -> Option<BuildRule> {
        self.build_plan.find_node(&label)
    }

    pub fn transitive_dependencies(&self, rule: &BuildRule) -> Vec<BuildRule> {
        rule.dependencies()
            .iter()
            .cloned()
            .flat_map(|dep_label| {
                let node = self.find_node(&dep_label).unwrap();
                let mut tran_deps = self.transitive_dependencies(&node);
                let mut transitive = vec![node];
                transitive.append(&mut tran_deps);
                transitive
            })
            .collect()
    }

    pub fn output_path(&self) -> PathBuf {
        self.artifact_root.clone().join("workspace")
    }

    pub fn cache_path(&self) -> PathBuf {
        self.artifact_root.clone().join("cache")
    }

    pub fn declare_output(&mut self, path: PathBuf) -> PathBuf {
        self.declared_outputs.push(path.clone());
        debug!("Declared output {:?}", &path);
        let actual_path = self.output_path().join(path);
        let parent = actual_path.parent().unwrap();
        fs::create_dir_all(parent).unwrap();
        actual_path
    }

    pub fn changed_inputs(&mut self, paths: &[PathBuf]) -> Option<Vec<(PathBuf, String)>> {
        let mut hasher = Sha1::new();
        let result: Option<Vec<(PathBuf, String)>> = paths
            .iter()
            .cloned()
            .map(|path| {
                let contents = fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("Truly expected {:?} to be a readable file. Was it changed since the build started?", path));
                hasher.input_str(&contents);
                let hex = hasher.result_str();
                hasher.reset();

                let cache_path = self.cache_path().join(&hex);
                if fs::metadata(&cache_path).is_ok() {
                    debug!("Cache hit for {:?} at {:?}", path, cache_path);
                    None
                } else {
                    debug!("No cache hit for {:?}", path);
                    Some((path, hex))
                }
            })
            .filter(Option::is_some)
            .collect();

        result.and_then(|x| if x.is_empty() { None } else { Some(x) })
    }

    pub fn copy(&mut self, files: &[PathBuf]) -> Result<(), anyhow::Error> {
        files
            .iter()
            .cloned()
            .map(|path| {
                let to = self.output_path().join(&path);
                debug!("Attempting to copy file from {:?} to {:?}", &path, &to);
                fs::copy(&path, &to)
                    .context(format!("Truly expected {:?} to be a readable file. Was it changed since the build started?", &path))
                    .map(|_| ())
            })
            .collect::<Result<(), anyhow::Error>>()
    }

    pub fn update_cache(
        &mut self,
        inputs: &[PathBuf],
        outputs: &[Artifact],
    ) -> Result<(), anyhow::Error> {
        let mut hasher = Sha1::new();
        let _ = inputs
            .iter()
            .cloned()
            .map(|path| {
                debug!("Attempting to create cache file for path: {:?}", &path);
                let contents = fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("Truly expected {:?} to be a readable file. Was it changed since the build started?", path));
                hasher.input_str(&contents);
                let hash = hasher.result_str();
                hasher.reset();

                let cache_path = self.cache_path().join(hash);
                fs::create_dir_all(&cache_path)
                    .context(format!("Could not create cache file for {:?} at {:?}", path, cache_path))
                    .map(|_| ())
            })
            .collect::<Result<(), anyhow::Error>>()?;

        outputs
            .iter()
            .cloned()
            .map(|artifact| {
                debug!("Caching build artifact: {:?}", &artifact);
                let hash = &artifact.compute_hash();
                let cache_path = self.cache_path().join(hash);
                debug!("Creating cache path: {:?}", &cache_path);
                fs::create_dir_all(&cache_path)
                    .context(format!(
                        "Could not create cache file for {:?} at {:?}",
                        artifact, cache_path
                    ))
                    .map(|_| ())?;

                let unique_outputs: HashSet<PathBuf> = artifact.outputs.iter().cloned().collect();

                for output in unique_outputs.iter() {
                    let workspace_file = self.output_path().join(output);
                    let cached_file = cache_path.join(output);
                    let cached_dir = &cached_file.parent().unwrap();
                    debug!("Creating artifact cache path: {:?}", &cached_dir);
                    fs::create_dir_all(&cached_dir)
                        .context(format!(
                            "Could not prepare directory for artifact {:?} into cache path: {:?}",
                            &output, &cached_dir
                        ))
                        .map(|_| ())?;
                    debug!("Moving artifact to cache: {:?}", &workspace_file);
                    fs::rename(&workspace_file, &cached_file)
                        .context(format!(
                            "Could not copy artifact {:?} into cache path: {:?}",
                            output, cached_file
                        ))
                        .map(|_| ())?;
                    let abs_path = fs::canonicalize(&cached_file)?;
                    debug!(
                        "Symlinking cached artifact to workspace: {:?} -> {:?}",
                        &abs_path, &workspace_file
                    );
                    symlink(&abs_path, &workspace_file).context(format!(
                        "Could not symlink {:#?} to {:#?}",
                        &abs_path, &workspace_file
                    ))?
                }

                Ok(())
            })
            .collect()
    }
}
