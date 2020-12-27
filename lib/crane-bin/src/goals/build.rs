use anyhow::Context;
use crane::build::{BuildPlan, BuildRunner};
use crane::label::Label;
use crane::workspace::Workspace;
use log::{debug, error, info};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "build",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "Build a target in this Workspace",
)]
struct BuildGoal {
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

impl BuildGoal {
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
