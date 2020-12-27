use anyhow::Context;
use crane::build::{BuildPlan, BuildRunner};
use crane::label::Label;
use crane::workspace::Workspace;
use log::{debug, error, info};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "fmt",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "formats all sources in the workspace"
)]
struct FmtGoal {}

impl FmtGoal {
    fn fmt(self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
