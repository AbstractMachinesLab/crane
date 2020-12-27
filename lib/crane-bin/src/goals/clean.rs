use anyhow::Context;
use crane::build::{BuildPlan, BuildRunner};
use crane::label::Label;
use crane::workspace::Workspace;
use log::{debug, error, info};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "clean",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "Cleans the entire workspace"
)]
struct CleanOpt {}

impl CleanOpt {
    fn clean(self) -> Result<(), anyhow::Error> {
        let workspace = Workspace::new().context("Could not create a workspace.")?;
        info!("Workspace: {}", &workspace.name());
        info!("Cleaning workspace...");
        workspace.clean()?;
        Ok(())
    }
}
