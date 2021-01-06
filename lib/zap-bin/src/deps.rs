use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "deps",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "dependency management"
)]
pub struct DepsGoal {}

impl DepsGoal {
    pub fn run(self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
