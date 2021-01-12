use super::BuildCache;
use anyhow::{anyhow, Context};
use log::*;
use std::collections::HashSet;
use std::path::PathBuf;
use zap_core::{ComputedTarget, Workspace};

/// A build Sandbox.
///
/// This is a spot where we isolate build nodes to execute them.
///
/// By the time we have created a Sandbox for a particular ComputedTarget, the node
/// is already _known_ to need building. This means that the node will already
/// be hashed.
///
/// The sandboxing steps are as folllows:
///
///   1. ensure the rule outputs are safe
///   2. prepare sandbox dir
///   3. copy dependences & inputs into sandbox
///   4. enter sandbox
///      5. build the node rule
///      6. validate rule's outputs
///   7. exit sandbox
///
///
pub struct Sandbox<'a> {
    /// The name of this sandbox, not to be confused by the `node.label()`, which
    /// is a Label.
    name: String,

    /// The node to be built.
    node: &'a ComputedTarget,

    /// The path to this sandbox on disk
    root: PathBuf,
    outputs_root: PathBuf,

    /// The outputs created during this build
    outputs: Vec<PathBuf>,

    status: ValidationStatus,
}

#[derive(Debug, Clone)]
pub enum ValidationStatus {
    Pending,
    Invalid {
        expected_and_present: Vec<PathBuf>,
        expected_but_missing: Vec<PathBuf>,
        unexpected_but_present: Vec<PathBuf>,
    },
    NoOutputs,
    Valid,
}

impl<'a> Sandbox<'a> {
    pub fn for_node(workspace: &Workspace, node: &'a ComputedTarget) -> Sandbox<'a> {
        let root = workspace.sandbox_root().join(node.hash());
        let outputs_root = workspace.local_outputs_root.clone();
        Sandbox {
            name: node.hash(),
            node,
            outputs: vec![],
            root,
            outputs_root,
            status: ValidationStatus::Pending,
        }
    }

    pub fn node(&self) -> &ComputedTarget {
        &self.node
    }

    pub fn root(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn outputs(&self) -> Vec<PathBuf> {
        self.outputs.clone()
    }

    fn scan_files(root: &PathBuf) -> Vec<PathBuf> {
        if root.is_dir() {
            std::fs::read_dir(root)
                .unwrap_or_else(|err| {
                    panic!("Could not read directory: {:?} due to {:?}", root, err)
                })
                .flat_map(|entry| {
                    let entry = entry.unwrap_or_else(|_| panic!("Could not read entry"));
                    let path = entry.path();
                    if path.is_dir() {
                        Sandbox::scan_files(&path)
                    } else {
                        vec![path]
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// Update the validation status of this sandbox by comparing the expected
    /// and actual outputs of this build rule.
    ///
    /// It is at this point that we enforce that every rule in the system will
    /// only create the outputs that it has declared.
    ///
    /// No undeclared outputs will be accepted, as they may collide with transitive
    /// dependency outputs that will be present in the sandbox.
    ///
    fn validate_outputs(&mut self) -> Result<(), anyhow::Error> {
        let node_inputs: HashSet<PathBuf> = self.node.srcs().iter().cloned().collect();
        let deps_inputs: HashSet<PathBuf> = self
            .node
            .deps()
            .iter()
            .flat_map(|n| n.outs.clone())
            .collect();

        debug!("Sandboxed Node Inputs: {:?}", &node_inputs);
        debug!("Sandboxed Deps Outputs: {:?}", &deps_inputs);

        let inputs: HashSet<PathBuf> = node_inputs.union(&deps_inputs).cloned().collect();

        let expected_outputs: HashSet<PathBuf> = self.node.outs().iter().cloned().collect();

        let all_outputs: HashSet<PathBuf> = Sandbox::scan_files(&self.root)
            .iter()
            .flat_map(|path| path.strip_prefix(&self.root))
            .map(|path| path.to_path_buf())
            .collect();

        // No outputs either expected or created, what did this rule do anyway?
        if expected_outputs.is_empty() || all_outputs.is_empty() {
            self.status = ValidationStatus::NoOutputs;
            return Ok(());
        }

        // All files found minus Inputs
        let actual_outputs: HashSet<PathBuf> = all_outputs
            .iter()
            .filter(|path| {
                !inputs.contains(&path.to_path_buf())
                    || expected_outputs.contains(&path.to_path_buf())
            })
            .cloned()
            .collect();

        debug!("Sandboxed Node Outputs: {:?}", &actual_outputs);

        let diff: Vec<&PathBuf> = expected_outputs.difference(&actual_outputs).collect();

        // No diff means we have the outputs we expected!
        if diff.is_empty() {
            self.status = ValidationStatus::Valid;
            self.outputs = actual_outputs.iter().cloned().collect();
        } else {
            let unexpected_but_present = diff.into_iter().cloned().collect();
            let expected_but_missing = expected_outputs
                .difference(&actual_outputs)
                .cloned()
                .collect();
            let expected_and_present = actual_outputs
                .intersection(&expected_outputs)
                .cloned()
                .collect();

            self.status = ValidationStatus::Invalid {
                expected_and_present,
                unexpected_but_present,
                expected_but_missing,
            }
        }

        Ok(())
    }

    fn prepare_sandbox_dir(&mut self) -> Result<PathBuf, anyhow::Error> {
        let current_dir = std::env::current_dir().context(format!(
            "Could not get the current directory while building sandbox for node {:?}",
            self.node.label().to_string(),
        ))?;

        let _ = std::fs::remove_dir_all(&self.root);
        std::fs::create_dir_all(&self.root)
            .context(format!(
                "Could not create sandbox directory for node {:?} at: {:?}",
                self.node.label().to_string(),
                &self.root
            ))
            .map(|_| ())?;

        debug!("Created sandbox at: {:?}", &self.root);

        Ok(current_dir)
    }

    fn copy_dependences(&mut self, build_cache: &BuildCache) -> Result<(), anyhow::Error> {
        // copy all the direct dependency outputs
        let deps: Vec<(PathBuf, PathBuf)> = self
            .node
            .deps()
            .iter()
            .flat_map(|dep| {
                let outs: Vec<(PathBuf, PathBuf)> = dep
                    .outs
                    .iter()
                    .map(|out| {
                        (
                            build_cache.absolute_path_by_hash(&dep.hash).join(&out),
                            self.root.join(&out),
                        )
                    })
                    .collect();
                outs
            })
            .collect();

        for (src, dst) in deps {
            if let Some(dst_parent) = &dst.parent() {
                std::fs::create_dir_all(dst_parent)
                    .context(format!(
                        "Could not create directory for transitive dependency {:?} at: {:?}",
                        &dst, &dst_parent
                    ))
                    .map(|_| ())?;
            };

            std::fs::copy(&src, &dst).context(format!(
            "When building {:?}, could not copy transitive dependency {:?} into sandbox at {:?}",
            self.node.label().to_string(),
            &src,
            &dst
        ))?;
            debug!("Copied {:?} to {:?}", &src, &dst);
        }

        Ok(())
    }

    fn copy_inputs(&mut self) -> Result<(), anyhow::Error> {
        // copy all inputs there
        let inputs = &self.node.srcs();
        for src in inputs {
            // find in cache
            let dst = self.root.join(&src);
            if let Some(dst_parent) = &dst.parent() {
                std::fs::create_dir_all(dst_parent)
                    .context(format!(
                        "Could not create directory for transitive dependency {:?} at: {:?}",
                        &dst, &dst_parent
                    ))
                    .map(|_| ())?;
            };
            std::fs::copy(&src, &dst).context(format!(
                "When building {:?}, could not copy input {:?} into sandbox at {:?}",
                self.name.to_string(),
                &src,
                &dst
            ))?;
            debug!("Copied {:?} to {:?}", &src, &dst);
        }
        Ok(())
    }

    fn enter_sandbox(&mut self) -> Result<(), anyhow::Error> {
        // move into the sandbox
        std::env::set_current_dir(&self.root).context(format!(
            "Could not move into the created sandbox at: {:?} when building {:?}",
            &self.root,
            self.node.label().to_string()
        ))?;
        debug!("Entered sandbox at: {:?}", &self.root);
        Ok(())
    }

    fn exit_sandbox(&mut self, working_directory: PathBuf) -> Result<(), anyhow::Error> {
        // move out of the sandbox
        std::env::set_current_dir(&working_directory).context(format!(
            "Could not move out of the created sandbox at: {:?} when building {:?}",
            &self.root,
            self.node.label().to_string()
        ))?;
        debug!("Exited sandbox at: {:?}", &self.root);
        Ok(())
    }

    pub fn clear_sandbox(&self) -> Result<(), anyhow::Error> {
        std::fs::remove_dir_all(&self.root).context(format!(
            "Could not clean sandbox for node {:?} at {:?}",
            self.node.label().to_string(),
            &self.root,
        ))
    }

    /// check that transitive output names and current rule output names
    /// do not collide.
    fn ensure_outputs_are_safe(&mut self) -> Result<(), anyhow::Error> {
        let output_set: HashSet<PathBuf> = self.node.outs().iter().cloned().collect();

        let dep_output_set: HashSet<PathBuf> = self
            .node
            .deps()
            .iter()
            .flat_map(|os| os.outs.clone())
            .collect();

        if !output_set.is_disjoint(&dep_output_set) {
            let overlapping_outputs = output_set.intersection(&dep_output_set);
            Err(anyhow!(
                "Oops, this rule would collide by creating outputs that would
          override dependency outputs: {:?}",
                overlapping_outputs
            ))
        } else {
            Ok(())
        }
    }

    fn promote_outputs(&mut self) -> Result<(), anyhow::Error> {
        for out in &self.outputs {
            let src = self.root.join(&out);
            let dst = self.outputs_root.join(&out);

            let dst_dir = dst.parent().context("Could not find parent dir")?;
            std::fs::create_dir_all(&dst_dir)
                .context(format!(
                    "Could not create sandbox directory for node {:?} at: {:?}",
                    self.node.label().to_string(),
                    &self.root
                ))
                .map(|_| ())?;

            trace!("Promoting {:?} to {:?}", src, dst);
            std::fs::copy(&src, &dst).context(format!(
                "When promoting outputs for target {:?}, could not copy {:?} into outputs at {:?}",
                self.node.label().to_string(),
                &src,
                &dst
            ))?;
        }
        Ok(())
    }

    /// Run a build rule within a sandboxed environment.
    ///
    /// NOTE(@ostera): wouldn't this be nice as a free monad?
    pub fn run(&mut self, build_cache: &BuildCache) -> Result<ValidationStatus, anyhow::Error> {
        self.ensure_outputs_are_safe()?;

        let working_directory = self.prepare_sandbox_dir()?;

        self.copy_dependences(&build_cache)?;

        self.copy_inputs()?;

        self.enter_sandbox()?;

        debug!("Executing build rule...");
        self.node.execute()?;
        debug!("Build rule executed successfully.");

        self.exit_sandbox(working_directory)?;

        self.validate_outputs()?;

        if let ValidationStatus::Valid = self.status {
            self.promote_outputs()?;
        }

        debug!(
            "Sandbox status updated to: {:?} {:?}",
            &self.status, &self.outputs
        );

        Ok(self.status.clone())
    }
}
