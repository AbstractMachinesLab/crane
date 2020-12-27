use anyhow::Context;
use crane::build::{BuildPlan, BuildRunner};
use crane::label::Label;
use crane::workspace::Workspace;
use log::{debug, error, info};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "lift",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "wraps a rebar3, mix, or erlang.mk project in a Crane build target"
)]
struct LiftGoal {}

impl LiftGoal {
    fn lift(self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
