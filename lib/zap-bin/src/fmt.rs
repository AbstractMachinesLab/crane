use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "fmt",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "formats all sources in the workspace"
)]
pub struct FmtGoal {}

impl FmtGoal {
    pub fn run(self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
