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
}

impl CacheGoal {
    pub async fn run(self) -> Result<(), anyhow::Error> {
        let t0 = std::time::Instant::now();

        let config = ZapConfig::new()?;

        std::fs::remove_dir_all(config.cache_root)?;

        let t1 = t0.elapsed().as_millis();
        println!("\x1B[1000D\x1B[K\r⚡ done in {}ms", t1);

        Ok(())
    }
}
