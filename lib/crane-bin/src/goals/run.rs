use anyhow::Context;
use crane::build::{BuildPlan, BuildRunner};
use crane::label::Label;
use crane::workspace::Workspace;
use log::{debug, error, info};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "run",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "Executes a runnable target"
)]
struct RunGoal {
    #[structopt(help = r"The target to run.

A path to a directory with a crane file, followed by a colon
and the name of the label to be built.

Example: //my/library:shell

NOTE: not all targets are runnable. Non-runnable targets will
build their dependencies and exit.
")]
    target: String,
}

impl RunGoal {
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
