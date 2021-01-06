use anyhow::*;
use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ZapConfig {
    zap_root: ProjectDirs,
    pub rules_root: PathBuf,
    pub toolchains_root: PathBuf,
    pub cache_root: PathBuf,
}

impl ZapConfig {
    pub fn new() -> Result<ZapConfig, anyhow::Error> {
        let zap_root = ProjectDirs::from("dev", "abstractmachines", "zap")
            .context("Could not figure out Zap project directories")?;

        let rules_root = zap_root.config_dir().join("rules");
        let toolchains_root = zap_root.config_dir().join("toolchains");

        let user_root = zap_root
            .cache_dir()
            .join(format!("_user_{}", whoami::username()));
        let cache_root = user_root.join("cache");

        std::fs::create_dir_all(zap_root.config_dir())?;
        std::fs::create_dir_all(zap_root.cache_dir())?;
        std::fs::create_dir_all(zap_root.data_dir())?;
        std::fs::create_dir_all(&rules_root)?;
        std::fs::create_dir_all(&toolchains_root)?;
        std::fs::create_dir_all(&cache_root)?;

        Ok(ZapConfig {
            zap_root,
            rules_root,
            toolchains_root,
            cache_root,
        })
    }
}
