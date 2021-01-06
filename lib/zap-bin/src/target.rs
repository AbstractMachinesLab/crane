use anyhow::*;
use log::*;
use std::path::PathBuf;
use structopt::StructOpt;
use zap_core::*;
use zap_project::*;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "target",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "managing targets"
)]
pub struct TargetGoal {
    #[structopt(subcommand, help = "the command to run")]
    cmd: Action,
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    setting = structopt::clap::AppSettings::ColoredHelp,
)]
enum Action {
    #[structopt(help = r"List all the workspace targets")]
    List,
}

impl TargetGoal {
    pub async fn run(self) -> Result<(), anyhow::Error> {
        let config = ZapConfig::new()?;
        let mut zap = ZapWorker::new(config)?;
        zap.load(&PathBuf::from(&".")).await?;

        match self.cmd {
            Action::List => self.list_targets(&mut zap),
        }
    }

    fn list_targets(&self, zap: &mut ZapWorker) -> Result<(), anyhow::Error> {
        let dep_graph = &mut zap.dep_graph;
        for target in dep_graph.target_names() {
            println!("{}", target);
        }
        Ok(())
    }
}
