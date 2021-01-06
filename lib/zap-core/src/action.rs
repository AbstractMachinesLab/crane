use anyhow::*;
use log::*;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub enum Action {
    Exec(ExecAction),
}

impl Action {
    pub fn exec(cmd: PathBuf) -> ExecAction {
        ExecAction { cmd, args: vec![] }
    }

    pub fn run(self) -> Result<(), anyhow::Error> {
        match self {
            Action::Exec(e) => e.run(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecAction {
    cmd: PathBuf,
    args: Vec<String>,
}

impl ExecAction {
    fn run(self) -> Result<(), anyhow::Error> {
        let mut cmd = Command::new(&self.cmd);
        let output = cmd
            .stdout(Stdio::piped())
            .args(&self.args)
            .output()
            .context(format!("Could not spawn {:?}", self.cmd))?;

        trace!(
            "Executed {:?} with status code: {}",
            &self.cmd,
            output.status.code().unwrap()
        );

        if output.status.success() {
            Ok(())
        } else {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            Err(anyhow!("Error running {:?}", self.cmd))
        }
    }

    pub fn args(&mut self, args: &[&str]) -> &mut ExecAction {
        for arg in args {
            self.args.push(arg.to_string());
        }
        self
    }

    pub fn build(self) -> Action {
        Action::Exec(self)
    }
}
