use anyhow::Context;
use std::path::PathBuf;
use structopt::StructOpt;
use zap_core::*;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "cache",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "manage the global cache",
)]
pub enum CacheGoal {
    #[structopt(help = r"Cleans the entire global cache.

NOTE: if this fixes your build, this is a bug. Please report it!
")]
    Purge,

    #[structopt(help = r"Clear a single target.

NOTE: if this fixes your build, this is a bug. Please report it!
")]
    Clear {
        #[structopt(help = r"The target to clean.

Must be a single target.
")]
        target: String,
    },
}

impl CacheGoal {
    pub async fn run(self) -> Result<(), anyhow::Error> {
        let config = ZapConfig::new()?;

        match self {
            CacheGoal::Purge => {
                std::fs::remove_dir_all(config.archive_root)
                    .context("Could not remove entire Zap cache")?;
                std::fs::remove_dir_all(config.cache_root)
                    .context("Could not remove entire Zap cache")
            }
            CacheGoal::Clear { target } => {
                let target: Label = target.into();
                let mut zap = ZapWorker::new(config)?;
                zap.load(&PathBuf::from(&".")).await?;
                zap.build_dep_graph()?;
                let dep_graph = &mut zap.dep_graph.scoped(&target)?.seal(
                    &zap.action_map,
                    &zap.output_map,
                    &mut zap.bs_ctx,
                )?;

                let node = dep_graph.find_node(&target).context(format!(
                    "Target {} was not found in the Build Graph",
                    target.to_string()
                ))?;

                let cache_entry = zap.config.cache_root.join(node.hash());

                std::fs::remove_dir_all(&cache_entry).context(format!(
                    "Could not remove cached target: {}",
                    target.to_string()
                ))
            }
        }
    }
}
