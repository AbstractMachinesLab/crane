use anyhow::*;
use std::path::PathBuf;
use structopt::StructOpt;
use zap_core::*;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "dep-graph",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "manage the dependency graph"
)]
pub struct DepGraphGoal {
    #[structopt(subcommand, help = "the command to run")]
    cmd: Action,
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    setting = structopt::clap::AppSettings::ColoredHelp,
)]
enum Action {
    Print {
        #[structopt(help = r"Print the scoped in GraphViz format.
    ")]
        target: String,
    },
}

impl DepGraphGoal {
    pub async fn run(self, config: ZapConfig) -> Result<(), anyhow::Error> {
        let mut zap = ZapWorker::new(config)?;
        zap.load(&PathBuf::from(&".")).await?;
        zap.build_dep_graph()?;

        match self.cmd {
            Action::Print { ref target } => self.print(&target, &mut zap),
        }
    }

    fn print(&self, target: &str, zap: &mut ZapWorker) -> Result<(), anyhow::Error> {
        let label: Label = target.into();
        let dep_graph = &mut zap.dep_graph.scoped(&label)?.seal(
            &zap.action_map,
            &zap.output_map,
            &mut zap.bs_ctx,
        )?;

        println!("{}", dep_graph.to_graphviz());

        Ok(())
    }
}
