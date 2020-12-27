use anyhow::Context;
use crane::build::{BuildPlan, BuildRunner};
use crane::label::Label;
use crane::workspace::Workspace;
use log::{debug, error, info};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "new",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "creates a new clean workspace"
)]
struct NewGoal {}

impl NewGoal {
    fn new(self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
