use log::*;
use structopt::StructOpt;
use zap_build::*;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "zap",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "A simple, fast, and correct build system for modern polyglot teams"
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
        let t0 = std::time::Instant::now();

        human_panic::setup_panic!(Metadata {
            name: "zap".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: "Leandro Ostera <leandro@abstractmachines.dev>".into(),
            homepage: "https://zap.build".into(),
        });

        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Info)
            .format_timestamp_micros()
            .format_module_path(false)
            .parse_env("ZAP_LOG")
            .try_init()
            .unwrap();
        let cmd = self.cmd.unwrap_or_else(|| Goal::Build(BuildGoal::all()));
        match cmd.run().await {
            Ok(()) => (),
            Err(err) => error!("{:?}", &err),
        };

        let t1 = t0.elapsed().as_millis();
        if !self.quiet {
            println!("\x1B[1000D\x1B[K\r⚡ done in {}ms", t1);
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
enum Goal {
    Build(BuildGoal),
    Cache(CacheGoal),
    DepGraph(DepGraphGoal),
    Rules(RulesGoal),
    Targets(TargetGoal),
    Toolchains(ToolchainGoal),
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
            Goal::Cache(x) => x.run().await,
            Goal::DepGraph(x) => x.run().await,
            Goal::Rules(x) => x.run().await,
            Goal::Targets(x) => x.run().await,
            Goal::Toolchains(x) => x.run().await,
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
