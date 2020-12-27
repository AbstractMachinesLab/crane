use std::path::PathBuf;

pub const WORKSPACE: &str = "Workspace.toml";

#[derive(Debug, Clone, Default)]
pub struct Workspace {
    name: String,
    root: PathBuf,
}

impl Workspace {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn crane_root(&self) -> PathBuf {
        self.root.join(".crane")
    }

    pub fn workspace_root(&self) -> PathBuf {
        self.crane_root().join("workspace")
    }

    pub fn cache_root(&self) -> PathBuf {
        self.crane_root().join("cache")
    }

    pub fn sandbox_root(&self) -> PathBuf {
        self.crane_root().join("sandbox")
    }
}
