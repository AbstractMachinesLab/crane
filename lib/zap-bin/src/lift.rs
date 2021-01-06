use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "lift",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "wraps a rebar3, mix, or erlang.mk project in a Zap build target"
)]
pub struct LiftGoal {}

impl LiftGoal {
    pub fn run(self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
