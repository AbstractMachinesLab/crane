use anyhow::*;
use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ZapConfig {
    project_dirs: ProjectDirs,

    /// The location of the global archive
    pub archive_root: PathBuf,

    /// The location of the global cache
    pub cache_root: PathBuf,

    /// The location of rule overrides.
    pub rules_root: PathBuf,

    /// The location of toolchain overrides.
    pub toolchains_root: PathBuf,

    /// The user running this command.
    pub user: String,
}

impl ZapConfig {
    /// NOTE: this probably should be built in bin/main.rs and passed on to the rest of the
    /// commands. That way we get a chance to configure all of this things and some others.

    pub fn new(home: Option<String>, user: Option<String>) -> Result<ZapConfig, anyhow::Error> {
        let user = user.unwrap_or_else(|| whoami::username());

        let project_dirs = ProjectDirs::from("dev", "abstractmachines", "zap")
            .context("Could not figure out Zap project directories")?;

        let config_dir = if let Some(ref root) = home {
            PathBuf::from(root)
        } else {
            project_dirs.config_dir().to_path_buf()
        };

        let cache_dir = if let Some(ref root) = home {
            PathBuf::from(root).join("cache")
        } else {
            project_dirs.cache_dir().to_path_buf()
        };

        let rules_root = config_dir.join("rules");
        let toolchains_root = config_dir.join("toolchains");

        let user_root = cache_dir.join(format!("_user_{}", user));
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
            user,
        })
    }
}
