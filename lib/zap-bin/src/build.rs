use anyhow::Context;
use log::*;
use std::io;
use std::io::Write;
use structopt::StructOpt;
use zap_build_engine::*;
use zap_core::*;
use zap_project::*;

use std::path::PathBuf;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "build",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "Build a target in this Workspace",
)]
pub struct BuildGoal {
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
            target: "//...".to_string(),
        }
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        let t0 = std::time::Instant::now();
        let target: Label = self.target.into();
        debug!("Host: {}", guess_host_triple::guess_host_triple().unwrap());
        debug!("Target: {}", &target.to_string());

        let config = ZapConfig::new()?;
        let mut zap = ZapWorker::new(config)?;
        zap.load(&PathBuf::from(&".")).await?;

        let mut runner = BuildRunner::new(zap);

        let name = if target.is_all() {
            "workspace".to_string()
        } else {
            target.to_string()
        };

        print!("🔨 Building {}...", name);
        io::stdout().flush().unwrap();

        runner.execute(&target)?;

        let t1 = t0.elapsed().as_millis();
        println!("\x1B[1000D\x1B[K\r⚡ done in {}ms", t1);

        Ok(())
    }
}
