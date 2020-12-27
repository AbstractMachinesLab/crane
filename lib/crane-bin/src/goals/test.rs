use anyhow::Context;
use crane::build::{BuildPlan, BuildRunner};
use crane::label::Label;
use crane::workspace::Workspace;
use log::{debug, error, info};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "test",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "Incrementally run one or many tests"
)]
struct TestGoal {
    #[structopt(
        help = r"The test target to run.

A path to a directory with a crane file, followed by a colon
and the name of the label to be built.

Example: //my/library:test_name

When unspecified, all tests will be run.

NOTE: not all targets are runnable. Non-test targets will
build their dependencies and exit.
",
        default = "//..."
    )]
    target: String,
}

impl TestGoal {
    fn test(self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
