use anyhow::*;
use log::*;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub enum Action {
    Exec(ExecAction),
    Copy(CopyAction),
    WriteFile(WriteFileAction),
}

impl Action {
    pub fn write_file(contents: String, dst: PathBuf) -> Action {
        Action::WriteFile(WriteFileAction { contents, dst })
    }

    pub fn copy(src: PathBuf, dst: PathBuf) -> Action {
        Action::Copy(CopyAction { src, dst })
    }

    pub fn exec(cmd: PathBuf) -> ExecAction {
        ExecAction {
            cmd,
            args: vec![],
            cwd: None,
        }
    }

    pub fn run(self) -> Result<(), anyhow::Error> {
        match self {
            Action::Exec(e) => e.run(),
            Action::Copy(e) => e.run(),
            Action::WriteFile(e) => e.run(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WriteFileAction {
    contents: String,
    dst: PathBuf,
}

impl WriteFileAction {
    fn run(self) -> Result<(), anyhow::Error> {
        if let Some(parent) = self.dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.dst, &self.contents)
            .map(|_| ())
            .context(format!("Could not run action {:#?}", &self))
    }
}

#[derive(Debug, Clone)]
pub struct CopyAction {
    src: PathBuf,
    dst: PathBuf,
}

impl CopyAction {
    fn run(self) -> Result<(), anyhow::Error> {
        if let Some(parent) = self.dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&self.src, &self.dst)
            .map(|_| ())
            .context(format!("Could not run action {:#?}", &self))
    }
}

#[derive(Debug, Clone)]
pub struct ExecAction {
    cmd: PathBuf,
    args: Vec<String>,
    cwd: Option<PathBuf>,
}

impl ExecAction {
    fn run(self) -> Result<(), anyhow::Error> {
        let mut cmd = Command::new(&self.cmd);
        cmd.stdout(Stdio::piped()).args(&self.args);
        if let Some(cwd) = self.cwd {
            cmd.current_dir(cwd);
        }

        trace!("Executing {:#?}", &cmd,);

        let output = cmd
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

    pub fn cwd(&mut self, cwd: &PathBuf) -> &mut ExecAction {
        self.cwd = Some(cwd.to_path_buf());
        self
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
