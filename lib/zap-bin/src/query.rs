use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "query",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "Query the Build Graph"
)]
pub struct QueryGoal {
    #[structopt(help = r"")]
    query: String,
}

impl QueryGoal {
    pub fn run(self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
