use super::{BuildNode, BuildRunner};
use crate::workspace::Workspace;
use anyhow::{Context, Error};

/// A build Sandbox.
///
/// This is a spot where we isolate build nodes to execute them.
///
/// By the time we have created a Sandbox for a particular BuildNode, the node
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
#[derive(Debug, Clone)]
pub struct Sandbox {
    /// The name of this sandbox
    name: String,

    /// The node to be built.
    node: BuildNode,

    /// The plan this node is a part of. It will be used to find the actual
    /// dependencies of the node above.
    build_plan: BuildPlan,

    /// The path to this sandbox on disk
    root: PathBuf,

    /// The outputs created during this build
    outputs: Vec<PathBuf>,

    status: ValidationStatus,
}

#[derive(Debug, Clone)]
pub enum ValidationStatus {
    Pending,
    Invalid {
        expected_but_missing: Vec<PathBuf>,
        unexpected_but_present: Vec<PathBuf>,
    },
    NoOutputs,
    Valid,
}

impl Sandbox {
    pub fn for_node(workspace: Workspace, node: BuildNode, build_plan: BuildPlan) -> Sandbox {
        Sandbox {
            build_plan,
            name: node.hash(),
            node,
            outputs: vec![],
            root: workspace.sandbox_root().join(node.hash()),
            status: ValidationStatus::Pending,
        }
    }

    /// Return the validation status of this sandbox by comparing the expected
    /// and actual outputs of this build rule.
    ///
    /// It is at this point that we enforce that every rule in the system will
    /// only create the outputs that it has declared.
    ///
    /// No undeclared outputs will be accepted, as they may collide with transitive
    /// dependency outputs that will be present in the sandbox.
    ///
    fn validate_outputs(&mut self) {
        let expected_outputs: HashSet<PathBuf> = self.node.rule().outputs().iter().collect();
        let actual_outputs: HashSet<PathBuf> = self.outputs.iter().collect();

        /// No outputs either expected or created, what did this rule do anyway?
        if expected_outputs.is_empty() && actual_outputs.is_empty() {
            self.status = ValidationStatus::NoOutputs;
        }

        let diff = actual_outputs.difference(expected_outputs);

        /// No diff means we have the outputs we expected!
        if diff.is_empty() {
            self.status = ValidationStatus::Valid;
        }

        self.status = ValidationStatus::Invalid {
            unexpected_but_present: diff.iter().collect(),
            expected_but_missing: expected_outputs.difference(actual_outputs).iter().collect(),
        }
    }

    fn prepare_sandbox_dir(&mut self) -> Result<(), anyhow::Error> {
        // remember where we are
        let current_dir = std::env::current_dir().context(format!(
            "Could not get the current directory while building sandbox for node {:?}",
            name.to_string(),
        ))?;

        // create .crane/sandbox/<sha>
        let sandbox_path = ctx.sandbox_path().join(hash);
        let _ = fs::remove_dir_all(&sandbox_path);
        fs::create_dir_all(&sandbox_path)
            .context(format!(
                "Could not create sandbox directory for node {:?} at: {:?}",
                name.to_string(),
                &sandbox_path
            ))
            .map(|_| ())?;

        debug!("Created sandbox at: {:?}", &sandbox_path);

        Ok(())
    }

    fn copy_dependences(&mut self) -> Result<(), anyhow::Error> {
        // copy all the direct dependency outputs
        let deps: Vec<(PathBuf, PathBuf)> = dep_nodes
            .iter()
            .flat_map(|dep| {
                let dep_hash = &dep.hash(ctx);
                let outs: Vec<(PathBuf, PathBuf)> = dep
                    .outputs(&ctx)
                    .iter()
                    .cloned()
                    .flat_map(|artifact| artifact.outputs)
                    .map(|out| {
                        (
                            std::fs::canonicalize(ctx.cache_path().join(&dep_hash).join(&out))
                                .unwrap(),
                            ctx.sandbox_path().join(&hash).join(&out),
                        )
                    })
                    .collect();
                outs
            })
            .collect();

        for (src, dst) in deps {
            if let Some(dst_parent) = &dst.parent() {
                fs::create_dir_all(dst_parent)
                    .context(format!(
                        "Could not create directory for transitive dependency {:?} at: {:?}",
                        &dst, &dst_parent
                    ))
                    .map(|_| ())?;
            };

            symlink(&src, &dst).context(format!(
            "When building {:?}, could not link transitive dependency {:?} into sandbox at {:?}",
            name.to_string(),
            &src,
            &dst
        ))?;
            debug!("Copied {:?} to {:?}", &src, &dst);
        }

        Ok(())
    }

    fn copy_inputs(&mut self) -> Result<(), anyhow::Error> {
        // copy all inputs there
        let inputs = &node.rule().inputs(&ctx);
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
        Ok(())
    }

    fn enter_sandbox(&mut self) -> Result<(), anyhow::Error> {
        // move into the sandbox
        std::env::set_current_dir(&sandbox_path).context(format!(
            "Could not move into the created sandbox at: {:?} when building {:?}",
            &sandbox_path,
            name.to_string()
        ))?;
        debug!("Entered sandbox at: {:?}", &sandbox_path);
        Ok(())
    }

    fn exit_sandbox(&mut self) -> Result<(), anyhow::Error> {
        // move out of the sandbox
        std::env::set_current_dir(&current_dir).context(format!(
            "Could not move out of the created sandbox at: {:?} when building {:?}",
            &sandbox_path,
            name.to_string()
        ))?;
        debug!("Exited sandbox at: {:?}", &sandbox_path);
        Ok(())
    }

    /// check that transitive output names and current rule output names
    /// do not collide.
    fn ensure_outputs_are_safe(&mut self) -> Result<(), anyhow::Error> {
        let dep_labels = self.node.rule().dependencies();
        let dep_nodes = self.build_plan.find_nodes(dep_labels);
        let dep_output_set = dep_nodes.iter().map(|os| os.outputs).collect();

        if !output_set.is_disjoint(dep_output_set) {
            let overlapping_outputs = output_set.intersection(dep_output_set);
            Error(anyhow!(
                "Oops, this rule would collide by creating outputs that would
          override dependency outputs: {:?}",
                overlapping_outputs
            ))
        }?;
        Ok(())
    }

    /// Run a build rule within a sandboxed environment.
    ///
    /// NOTE(@ostera): wouldn't this be nice as a free monad?
    pub fn run(&mut self, runner: &mut BuildRunner) -> Result<(), anyhow::Error> {
        self.ensure_outputs_are_safe()?;

        self.prepare_sandbox_dir()?;

        self.copy_dependences()?;

        self.copy_inputs()?;

        self.enter_sandbox()?;

        self.node.rule().build(ctx)?;

        self.validate_outputs();

        self.exit_sandbox()?;

        Ok(())
    }
}
