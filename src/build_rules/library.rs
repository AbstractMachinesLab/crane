use crate::build_plan::BuildContext;
use crate::model::target::Label;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Library {
    name: Label,
    files: Vec<PathBuf>,
    dependencies: Vec<Label>,
    outputs: Vec<PathBuf>,
    has_changed: bool,
}

impl Library {
    pub fn new(name: Label) -> Library {
        Library {
            name,
            files: vec![],
            dependencies: vec![],
            outputs: vec![],
            has_changed: false,
        }
    }

    pub fn set_name(&self, name: Label) -> Library {
        Library {
            name,
            ..self.clone()
        }
    }

    pub fn set_files(&self, files: Vec<PathBuf>) -> Library {
        Library {
            files,
            ..self.clone()
        }
    }

    pub fn set_dependencies(&self, dependencies: Vec<Label>) -> Library {
        Library {
            dependencies,
            ..self.clone()
        }
    }

    pub fn name(&self) -> Label {
        self.name.clone()
    }

    pub fn files(&self) -> Vec<PathBuf> {
        self.files.clone()
    }

    pub fn dependencies(&self) -> Vec<Label> {
        self.dependencies.clone()
    }

    pub fn outputs(&self, ctx: &BuildContext) -> Vec<PathBuf> {
        self.files
            .iter()
            .map(|file| ctx.path_in_context(file.with_extension("beam")))
            .collect()
    }

    pub fn has_changed(&self) -> bool {
        self.has_changed
    }

    pub fn declare_outputs(&self) -> Library {
        let outputs = self
            .files
            .iter()
            .map(|file| file.with_extension("beam"))
            .collect();
        Library {
            outputs,
            ..self.clone()
        }
    }

    pub fn execute(&self, ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        let beam_files: Vec<PathBuf> = self
            .files
            .iter()
            .cloned()
            .map(|f| ctx.declare_output(f.with_extension("beam")))
            .collect();
        ctx.toolchain().compile(&self.files, &beam_files[0]);
        Ok(())
    }
}
