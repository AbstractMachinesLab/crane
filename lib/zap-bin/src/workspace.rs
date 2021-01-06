use anyhow::*;
use std::path::PathBuf;
use structopt::StructOpt;
use zap_core::*;
use zap_project::*;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "workspace",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "workspace related goals"
)]
pub enum WorkspaceGoal {
    #[structopt(
        name = "info",
        setting = structopt::clap::AppSettings::ColoredHelp,
        about = "print information about this workspace"
    )]
    Info,
}

impl WorkspaceGoal {
    pub async fn run(self) -> Result<(), anyhow::Error> {
        match self {
            WorkspaceGoal::Info => {
                let config = ZapConfig::new()?;
                let mut zap = ZapWorker::new(config)?;
                zap.scan(&PathBuf::from(&"."))?;
                let workspace = zap.workspace();
                println!("Name: {:?}", workspace.name());
                println!("Workspace Root: {:?}", workspace.root());
                println!("Global Zap Directories:");
                println!("* Cache: {:?}", zap.config().cache_root);
                println!("* Rules: {:?}", zap.config().rules_root);
                println!("* Toolchains: {:?}", zap.config().toolchains_root);
                println!("Local Zap Directories:");
                println!("* Outputs: {:?}", workspace.outputs_root());
                println!("* Rules: {:?}", workspace.rules_root());
                println!("* Sandbox: {:?}", workspace.sandbox_root());
                println!("* Toolchains: {:?}", workspace.toolchains_root());
                println!("# of Targets: {:?}", workspace.targets().len());
                Ok(())
            }
        }
    }
}
