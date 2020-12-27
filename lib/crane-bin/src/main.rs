use anyhow::Context;
use crane::build::{BuildPlan, BuildRunner};
use crane::label::Label;
use crane::workspace::Workspace;
use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, error, info};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "crane",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "A multilanguage, incremental, scalable build system for the BEAM"
)]
struct Crane {
    #[structopt(short = "v", long = "verbose", help = "turn on verbosity")]
    verbose: bool,

    #[structopt(short = "q", long = "quiet", help = "turn off all logs")]
    quiet: bool,

    #[structopt(subcommand, help = "the command to run")]
    cmd: Goal,
}

impl Crane {
    fn run(self) {
        env_logger::init();
        match self.cmd.run() {
            Ok(()) => (),
            Err(err) => error!("{:?}", &err),
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
enum Goal {
    Build(BuildGoal),
    Run(RunGoal),
    Clean(CleanGoal),
}

impl Goal {
    fn run(self) -> Result<(), anyhow::Error> {
        match self {
            Goal::Build(b) => b.build(),
            Goal::Run(r) => r.run(),
            Goal::Clean(c) => c.clean(),
        }
    }
}

fn main() {
    Crane::from_args().run();
}
