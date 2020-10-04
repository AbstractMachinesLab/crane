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
        self.setup_logging();
        match self.cmd.run() {
            Ok(()) => (),
            Err(err) => error!("{:?}", &err),
        }
    }

    fn setup_logging(&self) {
        let colors_line = ColoredLevelConfig::new()
            .error(Color::Red)
            .warn(Color::Yellow)
            .info(Color::White)
            .debug(Color::BrightBlack)
            .trace(Color::BrightBlack);
        let colors_level = colors_line.clone().info(Color::Green);
        fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "{color_line}{date} {level}{color_line} :: {message}\x1B[0m",
                    color_line = format_args!(
                        "\x1B[{}m",
                        colors_line.get_color(&record.level()).to_fg_str()
                    ),
                    date = chrono::Local::now().format("%H:%M:%S"),
                    level = colors_level.color(record.level()),
                    message = message,
                ));
            })
            .level(match (self.verbose, self.quiet) {
                (_, true) => log::LevelFilter::Off,
                (true, false) => log::LevelFilter::Debug,
                (false, false) => log::LevelFilter::Info,
            })
            .level_for("pretty_colored", log::LevelFilter::Trace)
            .chain(std::io::stderr())
            .apply()
            .unwrap();
    }
}

#[derive(StructOpt, Debug, Clone)]
enum Goal {
    Build(BuildOpt),
    Run(RunOpt),
    Clean(CleanOpt),
}

impl Goal {
    fn run(self) -> Result<(), anyhow::Error> {
        match self {
            Goal::Build(opts) => opts.build(),
            Goal::Run(opts) => opts.run(),
            Goal::Clean(opts) => opts.clean(),
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "build",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "Build a target in this Workspace",
)]
struct BuildOpt {
    #[structopt(
        short = "p",
        long = "print-graph",
        help = "prints the build graph in GraphViz format"
    )]
    print_graph: bool,

    #[structopt(
        help = r"The target to build.

A path to a directory with a crane file, followed by a colon
and the name of the label to be built.

Example: //my/library:lib

Use //... to build the entire project.
",
        default_value = "//..."
    )]
    target: String,
}

impl BuildOpt {
    fn build(self) -> Result<(), anyhow::Error> {
        let t0 = std::time::Instant::now();
        let target: Label = self.target.into();
        debug!("Host: {}", guess_host_triple::guess_host_triple().unwrap());
        debug!("Target: {}", &target.to_string());

        let workspace = Workspace::new().context("Could not create a workspace.")?;
        debug!("Workspace: {}", &workspace.name());

        info!("Planning build...");
        let build_plan = BuildPlan::from_rules(workspace.rules())?.scoped(target.clone())?;

        if self.print_graph {
            info!("Printing build graph as GraphViz Dot...");
            println!("{}", build_plan.to_graphviz());
            let t1 = t0.elapsed().as_millis();
            info!("Printed {} in {}ms", target.to_string(), t1);
        } else {
            info!("Readying toolchains: {:?}", &build_plan.toolchains_in_use());
            let mut runner = BuildRunner::new(workspace, build_plan);
            let _ = &mut runner.ready_toolchains()?;

            info!("Building target: {}", &target.to_string());
            let artifacts = runner.build()?;

            let t1 = t0.elapsed().as_millis();
            info!("Built {} artifacts in {}ms", artifacts, t1);
        }
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "run",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "Executes a runnable target"
)]
struct RunOpt {
    #[structopt(help = r"The target to run.

A path to a directory with a crane file, followed by a colon
and the name of the label to be built.

Example: //my/library:shell

NOTE: not all targets are runnable. Non-runnable targets will
build their dependencies and exit.
")]
    target: String,
}

impl RunOpt {
    fn run(self) -> Result<(), anyhow::Error> {
        let t0 = std::time::Instant::now();
        let workspace = Workspace::new().context("Could not create a workspace.")?;
        let target: Label = self.target.into();

        if target.is_all() {
            error!("You must specify a single target to run.");
            return Ok(());
        }

        info!("Workspace: {}", &workspace.name());
        info!("Target: {}", &target.to_string());

        info!("Planning build...");
        let build_plan = BuildPlan::from_rules(workspace.rules())?.scoped(target.clone())?;
        info!("Readying toolchains: {:?}", &build_plan.toolchains_in_use());
        let mut runner = BuildRunner::new(workspace, build_plan);
        let _ = &mut runner.ready_toolchains()?;

        info!("Building target: {}", &target.to_string());
        let artifacts = runner.build()?;
        let t1 = t0.elapsed().as_millis();
        info!("Built {} artifacts in {}ms", artifacts, t1);
        info!("Running target:");
        let _ = &mut runner.run()?;

        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "clean",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "Cleans the entire workspace"
)]
struct CleanOpt {}

impl CleanOpt {
    fn clean(self) -> Result<(), anyhow::Error> {
        let workspace = Workspace::new().context("Could not create a workspace.")?;
        info!("Workspace: {}", &workspace.name());
        info!("Cleaning workspace...");
        workspace.clean()?;
        Ok(())
    }
}

fn main() {
    Crane::from_args().run();
}
