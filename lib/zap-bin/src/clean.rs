use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "clean",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "Cleans the entire workspace"
)]
pub struct CleanGoal {}

impl CleanGoal {
    pub fn run(self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
