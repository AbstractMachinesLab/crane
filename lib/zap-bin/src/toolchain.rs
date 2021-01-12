use anyhow::*;
use std::path::PathBuf;
use structopt::StructOpt;
use zap_core::*;
use zap_project::*;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "toolchain",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "manage language toolchains"
)]
pub enum ToolchainGoal {
    #[structopt(
        name = "list-archives",
        setting = structopt::clap::AppSettings::ColoredHelp,
        about = "lists toolchain archives"
    )]
    ListArchives,

    #[structopt(
        name = "list",
        setting = structopt::clap::AppSettings::ColoredHelp,
        about = "lists active toolchain"
    )]
    ListToolchains,
}

impl ToolchainGoal {
    pub async fn run(self) -> Result<(), anyhow::Error> {
        let config = ZapConfig::new()?;
        let mut zap = ZapWorker::new(config)?;
        zap.load(&PathBuf::from(&".")).await?;

        match self {
            ToolchainGoal::ListArchives => self.list_archives(&mut zap),
            ToolchainGoal::ListToolchains => self.list_toolchains(&mut zap),
        }
    }

    pub fn list_archives(&self, zap: &mut ZapWorker) -> Result<(), anyhow::Error> {
        let toolchain_manager = zap.toolchain_manager();
        let archives = toolchain_manager.read().unwrap().archives();

        if archives.is_empty() {
            println!("No archives configured.");
        } else {
            println!("Configured Toolchain Archives: ");
            for archive in archives {
                println!(
                    r#"* {name}:
  - url: {url}
  - hash: {hash}
"#,
                    name = archive.name().to_string(),
                    url = archive.url(),
                    hash = archive.hash()
                );
            }
        }
        Ok(())
    }

    pub fn list_toolchains(&self, zap: &mut ZapWorker) -> Result<(), anyhow::Error> {
        let toolchain_manager = zap.toolchain_manager();
        let toolchains = toolchain_manager.read().unwrap().toolchains();

        if toolchains.is_empty() {
            println!("No active toolchains.");
        } else {
            println!("Active Toolchains: ");
            for toolchain in toolchains {
                println!(r#"* {}"#, toolchain.label().to_string());
            }
        }
        Ok(())
    }
}
