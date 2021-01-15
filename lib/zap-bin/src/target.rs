use anyhow::*;
use std::path::PathBuf;
use structopt::StructOpt;
use zap_core::*;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "targets",
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
    pub async fn run(self, config: ZapConfig) -> Result<(), anyhow::Error> {
        let mut zap = ZapWorker::new(config)?;
        zap.load(&PathBuf::from(&".")).await?;
        zap.build_dep_graph()?;

        match self.cmd {
            Action::List => self.list_targets(&mut zap),
        }
    }

    fn list_targets(&self, zap: &mut ZapWorker) -> Result<(), anyhow::Error> {
        let dep_graph = &mut zap.dep_graph;
        let mut targets = dep_graph.target_names();
        targets.sort();
        for target in targets {
            println!("{}", target);
        }
        Ok(())
    }
}
