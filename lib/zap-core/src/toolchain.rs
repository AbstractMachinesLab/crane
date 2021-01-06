use super::*;

/*
 *
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
pub trait Toolchain: CloneUnsized + Debug {
    fn tools(&self) -> &HashMap<String, PathBuf>;

    fn into_target(&self) -> &dyn Target;
}

pub trait CloneUnsized {
    fn clone_boxed(&self) -> Box<dyn Toolchain>;
}

impl<T: Clone + Toolchain + 'static> CloneUnsized for T {
    fn clone_boxed(&self) -> Box<dyn Toolchain> {
        Box::new(self.clone())
    }
}

*/

#[derive(Default, Clone, Debug)]
pub struct Toolchain {}

#[derive(Default, Clone, Debug)]
pub struct ToolchainManager {}

impl ToolchainManager {
    pub fn new() -> ToolchainManager {
        ToolchainManager {}
    }

    pub fn toolchains_as_targets(&self) -> Vec<Target> {
        vec![]
        /*
        self.toolchains
            .values()
            .into_iter()
            .map(|t| t.into_target().clone_boxed())
            .collect()
            */
    }

    pub fn register(&mut self, _toolchain: Toolchain) -> &mut ToolchainManager {
        /* self.toolchains
            .insert(toolchain.into_target().label().clone(), toolchain);
        o*/
        self
    }

    pub fn get(&self, _label: &Label) -> &Toolchain {
        todo!();
        // &self.toolchains.get(&label).unwrap()
    }
}
