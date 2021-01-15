use log::*;
use std::convert::TryInto;
use structopt::StructOpt;
use zap_build::*;
use zap_core::*;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "zap",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "A simple, fast, and correct build system for modern polyglot teams"
)]
struct Zap {
    #[structopt(subcommand, help = "the command to run")]
    cmd: Option<Goal>,

    #[structopt(short = "v", long = "verbose", help = "turn on verbosity")]
    verbose: bool,

    #[structopt(short = "q", long = "quiet", help = "turn off all logs")]
    quiet: bool,

    #[structopt(
        long = "user",
        help = "the user running this command. This will default to $USER"
    )]
    user: Option<String>,

    #[structopt(
        long = "zap-home",
        help = "the root directory for the Zap global configuration"
    )]
    zap_home: Option<String>,
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

        let config = match self.clone().try_into() {
            Ok(config) => config,
            Err(err) => panic!("{:?}", &err),
        };

        let cmd = self.cmd.unwrap_or_else(|| Goal::Build(BuildGoal::all()));
        match cmd.run(config).await {
            Ok(()) => (),
            Err(err) => error!("{:?}", &err),
        };

        let t1 = t0.elapsed().as_millis();
        if !self.quiet {
            println!("\x1B[1000D\x1B[K\r⚡ done in {}ms", t1);
        }
    }
}

impl TryInto<ZapConfig> for Zap {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<ZapConfig, anyhow::Error> {
        ZapConfig::new(self.zap_home.clone(), self.user.clone())
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
    async fn run(self, config: ZapConfig) -> Result<(), anyhow::Error> {
        match self {
            Goal::Build(x) => x.run(config).await,
            Goal::Cache(x) => x.run(config).await,
            Goal::DepGraph(x) => x.run(config).await,
            Goal::Rules(x) => x.run(config).await,
            Goal::Targets(x) => x.run(config).await,
            Goal::Toolchains(x) => x.run(config).await,
            Goal::Workspace(x) => x.run(config).await,
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
