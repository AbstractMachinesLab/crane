use crate::model::target::Label;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Test {
    pub name: Label,
    pub file: PathBuf,
    pub dependencies: Vec<Label>,
}

impl Test {
    pub fn build(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    pub fn run(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    pub fn outputs(&self) -> Vec<PathBuf> {
        vec![]
    }

    pub fn inputs(&self) -> Vec<PathBuf> {
        vec![self.file.clone()]
    }

    pub fn dependencies(&self) -> Vec<Label> {
        self.dependencies.clone()
    }
}
