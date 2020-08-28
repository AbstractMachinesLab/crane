use crate::build_artifact::Artifact;
use crate::build_context::BuildContext;
use crate::build_rules::build_rule::BuildRule;
use crate::model::target::Label;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Shell {
    name: Label,
    dependencies: Vec<Label>,
}

impl Shell {
    pub fn new(name: Label) -> Shell {
        Shell {
            name,
            dependencies: vec![],
        }
    }

    pub fn set_name(&self, name: Label) -> Shell {
        Shell {
            name,
            ..self.clone()
        }
    }

    pub fn set_dependencies(&self, dependencies: Vec<Label>) -> Shell {
        Shell {
            dependencies,
            ..self.clone()
        }
    }

    pub fn name(&self) -> Label {
        self.name.clone()
    }

    pub fn dependencies(&self) -> Vec<Label> {
        self.dependencies.clone()
    }

    pub fn inputs(&self) -> Vec<PathBuf> {
        vec![]
    }

    pub fn outputs(&self, _ctx: &BuildContext) -> Vec<Artifact> {
        vec![]
    }

    pub fn run(&self, ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        let wrapped = BuildRule::Shell(self.clone());
        let code_paths: HashSet<PathBuf> = ctx
            .transitive_dependencies(&wrapped)
            .iter()
            .flat_map(|dep| dep.outputs(&ctx))
            .flat_map(|artifact| artifact.outputs)
            .map(|path| path.parent().unwrap().to_path_buf())
            .map(|path| ctx.output_path().join(path))
            .collect();

        ctx.toolchain().shell(&code_paths.into_iter().collect())
    }

    pub fn build(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
