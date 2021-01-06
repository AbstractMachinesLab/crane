use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "test",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "Incrementally run one or many tests"
)]
pub struct TestGoal {
    #[structopt(
        help = r"The test target to run.

A path to a directory with a zap file, followed by a colon
and the name of the label to be built.

Example: //my/library:test_name

When unspecified, all tests will be run.

NOTE: not all targets are runnable. Non-test targets will
build their dependencies and exit.
",
        default_value = "//..."
    )]
    target: String,
}

impl TestGoal {
    pub fn run(self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
