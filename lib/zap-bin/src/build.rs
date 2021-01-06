use anyhow::Context;
use zap_build_engine::BuildRunner;
use zap_core::{DepGraph, Label};
use zap_project::WorkspaceScanner;
use log::*;
use structopt::StructOpt;

use std::path::PathBuf;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "build",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "Build a target in this Workspace",
)]
pub struct BuildGoal {
    #[structopt(
        short = "p",
        long = "print-graph",
        help = "prints the build graph in GraphViz format"
    )]
    print_graph: bool,

    #[structopt(
        help = r"The target to build.

A path to a directory with a zap file, followed by a colon
and the name of the label to be built.

Example: //my/library:lib

Use //... to build the entire project.
",
        default_value = "//..."
    )]
    target: String,
}

impl BuildGoal {
    pub fn all() -> BuildGoal {
        BuildGoal {
            print_graph: false,
            target: "//...".to_string(),
        }
    }

    pub fn run(self) -> Result<(), anyhow::Error> {
        let t0 = std::time::Instant::now();
        let target: Label = self.target.into();
        debug!("Host: {}", guess_host_triple::guess_host_triple().unwrap());
        debug!("Target: {}", &target.to_string());

        let root = PathBuf::from(&".");
        let workspace = WorkspaceScanner::scan(&root).context("Could not create a workspace.")?;
        debug!("Workspace: {}", &workspace.name());

        info!("Planning build...");
        let dep_graph = DepGraph::from_targets(&workspace.targets())?.scoped(target.clone())?;

        if self.print_graph {
            info!("Printing build graph as GraphViz Dot...");
            println!("{}", dep_graph.to_graphviz());
            let t1 = t0.elapsed().as_millis();
            info!("Printed {} in {}ms", target.to_string(), t1);
        } else {
            let mut runner = BuildRunner::new(workspace, dep_graph);

            info!("Building target: {}", &target.to_string());
            let artifacts = runner.execute()?;

            let t1 = t0.elapsed().as_millis();
            info!("Built {} artifacts in {}ms", artifacts, t1);
        }
        Ok(())
    }
}
