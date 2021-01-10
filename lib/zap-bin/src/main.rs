use log::*;
use structopt::StructOpt;
use zap_bin::*;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "zap",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "A multilanguage, incremental, scalable build system"
)]
struct Zap {
    #[structopt(short = "v", long = "verbose", help = "turn on verbosity")]
    verbose: bool,

    #[structopt(short = "q", long = "quiet", help = "turn off all logs")]
    quiet: bool,

    #[structopt(subcommand, help = "the command to run")]
    cmd: Option<Goal>,
}

impl Zap {
    async fn run(self) {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Info)
            .format_module_path(false)
            .parse_env("ZAP_LOG")
            .try_init()
            .unwrap();
        let cmd = self.cmd.unwrap_or(Goal::Build(BuildGoal::all()));
        match cmd.run().await {
            Ok(()) => (),
            Err(err) => error!("{:?}", &err),
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
enum Goal {
    Build(BuildGoal),
    Depgraph(DepGraphGoal),
    Rules(RulesGoal),
    Target(TargetGoal),
    Toolchain(ToolchainGoal),
    Workspace(WorkspaceGoal),
    // Clean(CleanGoal),
    // Deps(DepsGoal),
    // Fmt(FmtGoal),
    // Lift(LiftGoal),
    // New(NewGoal),
    // Query(QueryGoal),
    // Run(RunGoal),
    // Test(TestGoal),
}

impl Goal {
    async fn run(self) -> Result<(), anyhow::Error> {
        match self {
            Goal::Build(x) => x.run().await,
            Goal::Depgraph(x) => x.run().await,
            Goal::Rules(x) => x.run().await,
            Goal::Target(x) => x.run().await,
            Goal::Toolchain(x) => x.run().await,
            Goal::Workspace(x) => x.run().await,
            // Goal::Clean(x) => x.run(),
            // Goal::Deps(x) => x.run(),
            // Goal::Fmt(x) => x.run(),
            // Goal::Lift(x) => x.run(),
            // Goal::New(x) => x.run(),
            // Goal::Query(x) => x.run(),
            // Goal::Run(x) => x.run(),
            // Goal::Test(x) => x.run(),
        }
    }
}

#[tokio::main]
async fn main() {
    Zap::from_args().run().await;
}
