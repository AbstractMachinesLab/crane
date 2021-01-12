use super::*;
use anyhow::*;
use std::path::PathBuf;

pub const WORKSPACE: &str = "Workspace.toml";

#[derive(Clone, Default, Debug)]
pub struct Workspace {
    name: String,
    pub local_outputs_root: PathBuf,
    pub local_rules_root: PathBuf,
    pub local_sandbox_root: PathBuf,
    pub local_toolchains_root: PathBuf,
    pub local_zap_root: PathBuf,
    pub workspace_root: PathBuf,
    targets: Vec<Target>,
}

impl Workspace {
    pub fn new(name: String, workspace_root: &PathBuf) -> Result<Workspace, anyhow::Error> {
        let workspace_root = workspace_root.to_path_buf();
        let local_zap_root = workspace_root.join(".zap");
        let local_rules_root = local_zap_root.join("rules");
        let local_toolchains_root = local_zap_root.join("toolchains");
        let local_sandbox_root = local_zap_root.join("sandbox");
        let local_outputs_root = local_zap_root.join("outputs");

        let workspace = Workspace {
            local_outputs_root,
            local_rules_root,
            local_sandbox_root,
            local_toolchains_root,
            local_zap_root,
            name,
            targets: vec![],
            workspace_root,
        };

        workspace.ensure_dirs()?;
        workspace.setup_links()?;

        Ok(workspace)
    }

    fn ensure_dirs(&self) -> Result<(), anyhow::Error> {
        std::fs::create_dir_all(&self.local_zap_root)?;
        std::fs::create_dir_all(&self.local_rules_root)?;
        std::fs::create_dir_all(&self.local_toolchains_root)?;
        std::fs::create_dir_all(&self.local_sandbox_root)?;
        std::fs::create_dir_all(&self.local_outputs_root)?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn setup_links(&self) -> Result<(), anyhow::Error> {
        let _ = std::fs::remove_file(self.workspace_root.join("zap-outputs"));
        match std::os::windows::fs::symlink_dir(
            self.local_outputs_root.clone(),
            self.workspace_root.join("zap-outputs"),
        ) {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            err => Err(anyhow!("Could not create symlink because of: {:?}", err)),
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn setup_links(&self) -> Result<(), anyhow::Error> {
        let _ = std::fs::remove_file(self.workspace_root.join("zap-outputs"));
        match std::os::unix::fs::symlink(
            self.local_outputs_root.clone(),
            self.workspace_root.join("zap-outputs"),
        ) {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            err => Err(anyhow!("Could not create symlink because of: {:?}", err)),
        }
    }

    pub fn with_targets(&mut self, targets: Vec<Target>) -> &mut Workspace {
        self.targets = targets;
        self
    }

    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn root(&self) -> &PathBuf {
        &self.workspace_root
    }

    pub fn sandbox_root(&self) -> &PathBuf {
        &self.local_sandbox_root
    }

    pub fn rules_root(&self) -> &PathBuf {
        &self.local_rules_root
    }

    pub fn toolchains_root(&self) -> &PathBuf {
        &self.local_toolchains_root
    }

    pub fn outputs_root(&self) -> &PathBuf {
        &self.local_outputs_root
    }
}
