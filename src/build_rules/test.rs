use crate::model::target::Label;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Test {
    pub name: Label,
    pub file: String,
    pub dependencies: Vec<Label>,
}

impl Test {
    pub fn execute(&self) -> Result<(), anyhow::Error> {
        println!("{:?}", &self);
        Ok(())
    }

    pub fn outputs(&self) -> Vec<PathBuf> {
        vec![]
    }

    pub fn dependencies(&self) -> Vec<Label> {
        self.dependencies.clone()
    }
}
