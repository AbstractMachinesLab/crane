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
        fs::create_dir_all(ctx.workspace_path()).unwrap();
        fs::create_dir_all(ctx.cache_path()).unwrap();
        fs::create_dir_all(ctx.sandbox_path()).unwrap();
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

    pub fn workspace_path(&self) -> PathBuf {
        self.artifact_root.clone().join("workspace")
    }

    pub fn cache_path(&self) -> PathBuf {
        self.artifact_root.clone().join("cache")
    }

    pub fn sandbox_path(&self) -> PathBuf {
        self.artifact_root.clone().join("sandbox")
    }

    /// Determine if a given node has been cached already or not.
    ///
    /// This is based on hash of the node (see `BuildRule::hash`).
    ///
    /// FIXME: check if the expected hashes of the inputs match the actual
    /// hash of the files to determine if the cache is corrupted.
    pub fn is_cached(&mut self, node: &BuildRule) -> Result<bool, anyhow::Error> {
        let name = &node.name();
        let hash = &node.hash(self);

        let cache_path = self.cache_path().join(&hash);
        let is_cached = fs::metadata(&cache_path).is_ok();

        if is_cached {
            debug!("Cache hit for {:?} at {:?}", name, cache_path);
        } else {
            debug!("No cache hit for {:?}", name);
        }

        Ok(is_cached)
    }

    /// Run a build rule within a sandboxed environment.
    ///
    /// FIXME: turn this into a BuildSandbox struct.
    pub fn run_in_sandbox(&mut self, node: &mut BuildRule) -> Result<(), anyhow::Error> {
        let name = &node.name();

        let current_dir = std::env::current_dir().context(format!(
            "Could not get the current directory while building sandbox for node {:?}",
            name.to_string(),
        ))?;

        let hash = &node.hash(self);
        debug!("Rule {:?} hashed as {:?}", &name, &hash);

        // create .crane/sandbox/<sha>
        let sandbox_path = self.sandbox_path().join(hash);
        fs::create_dir_all(&sandbox_path)
            .context(format!(
                "Could not create sandbox directory for node {:?} at: {:?}",
                name.to_string(),
                &sandbox_path
            ))
            .map(|_| ())?;
        debug!("Created sandbox at: {:?}", &sandbox_path);

        // copy all inputs there
        let inputs = &node.inputs(&self);
        for src in inputs {
            // find in cache
            let dst = sandbox_path.join(&src);
            let src = name.path().join(&src);
            std::fs::copy(&src, &dst).context(format!(
                "When building {:?}, could not copy input {:?} into sandbox at {:?}",
                name.to_string(),
                &src,
                &dst
            ))?;
            debug!("Copied {:?} to {:?}", &src, &dst);
        }

        // move into the sandbox
        std::env::set_current_dir(&sandbox_path).context(format!(
            "Could not move into the created sandbox at: {:?} when building {:?}",
            &sandbox_path,
            name.to_string()
        ))?;
        debug!("Entered sandbox at: {:?}", &sandbox_path);

        // run rule in that folder
        node.build(self)?;

        // list all the files
        // expect inputs + outputs
        // if there are more files, panic!

        // move out of the sandbox
        std::env::set_current_dir(&current_dir).context(format!(
            "Could not move out of the created sandbox at: {:?} when building {:?}",
            &sandbox_path,
            name.to_string()
        ))?;
        debug!("Exited sandbox at: {:?}", &sandbox_path);

        // update cache
        let cache_path = self.cache_path().join(hash);
        let workspace_path = self.workspace_path();

        let result = node
            .outputs(self)
            .iter()
            .cloned()
            .map(|artifact| {
                debug!("Caching build artifact: {:?}", &artifact);
                let unique_outputs: HashSet<PathBuf> = artifact.outputs.iter().cloned().collect();
                for output in unique_outputs.iter() {
                    let cached_file = cache_path.join(output);
                    let cached_dir = &cached_file.parent().unwrap();
                    debug!("Creating artifact cache path: {:?}", &cached_dir);
                    fs::create_dir_all(&cached_dir)
                        .context(format!(
                            "Could not prepare directory for artifact {:?} into cache path: {:?}",
                            &output, &cached_dir
                        ))
                        .map(|_| ())?;
                    let sandboxed_file = sandbox_path.join(output);
                    debug!(
                        "Moving artifact from sandbox path {:?} to cache path {:?}",
                        &sandboxed_file, &cached_file
                    );
                    fs::rename(&sandboxed_file, &cached_file)
                        .context(format!(
                            "Could not move artifact {:?} into cache path: {:?}",
                            sandboxed_file, cached_file
                        ))
                        .map(|_| ())?;
                    let abs_path = fs::canonicalize(&cached_file)?;
                    let workspace_file = workspace_path.join(output);
                    fs::create_dir_all(&workspace_file.parent().unwrap())
                        .context(format!(
                            "Could not create workspace directory for {:?}",
                            &workspace_path
                        ))
                        .map(|_| ())?;
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
            .collect::<Result<(), anyhow::Error>>();

        if let Err(_) = result {
            fs::remove_dir_all(&cache_path)?;
        }
        result?;

        // we're good!
        Ok(())
    }
}
