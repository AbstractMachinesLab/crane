use anyhow::*;
use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ZapConfig {
    project_dirs: ProjectDirs,
    pub archive_root: PathBuf,
    pub cache_root: PathBuf,
    pub rules_root: PathBuf,
    pub toolchains_root: PathBuf,
}

impl ZapConfig {
    pub fn new() -> Result<ZapConfig, anyhow::Error> {
        let project_dirs = ProjectDirs::from("dev", "abstractmachines", "zap")
            .context("Could not figure out Zap project directories")?;

        let config_dir = if let Ok(root) = std::env::var("ZAP_HOME") {
            PathBuf::from(root)
        } else {
            project_dirs.config_dir().to_path_buf()
        };

        let cache_dir = if let Ok(root) = std::env::var("ZAP_HOME") {
            PathBuf::from(root).join("cache")
        } else {
            project_dirs.cache_dir().to_path_buf()
        };

        let rules_root = config_dir.join("rules");
        let toolchains_root = config_dir.join("toolchains");

        let user_root = cache_dir.join(format!("_user_{}", whoami::username()));
        let cache_root = user_root.join("cache");
        let archive_root = user_root.join("archive");

        std::fs::create_dir_all(&rules_root)?;
        std::fs::create_dir_all(&toolchains_root)?;
        std::fs::create_dir_all(&cache_root)?;
        std::fs::create_dir_all(&archive_root)?;

        Ok(ZapConfig {
            archive_root,
            cache_root,
            project_dirs,
            rules_root,
            toolchains_root,
        })
    }
}
