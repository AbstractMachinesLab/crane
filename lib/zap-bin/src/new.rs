use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "new",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "creates a new clean workspace"
)]
pub struct NewGoal {}

impl NewGoal {
    pub fn run(self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
