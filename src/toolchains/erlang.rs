use anyhow::Context;
use log::debug;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct Toolchain {
    language: String,
    version: String,
}

impl Default for Toolchain {
    fn default() -> Toolchain {
        Toolchain {
            language: "erlang".to_string(),
            version: "23".to_string(),
        }
    }
}

impl Toolchain {
    pub fn language(&self) -> &String {
        &self.language
    }

    pub fn version(&self) -> &String {
        &self.version
    }

    pub fn shell(self, code_paths: &Vec<PathBuf>) -> Result<(), anyhow::Error> {
        debug!("Starting shell with dependencies: {:?}", &code_paths);
        let mut code_paths: Vec<String> = code_paths
            .iter()
            .map(|p| String::from(p.to_str().unwrap()))
            .collect();
        let mut args = vec![String::from("-pa")];
        args.append(&mut code_paths);

        let mut cmd = Command::new("erl");
        let stdout = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(args.as_slice())
            .spawn()
            .context("Could not spawn erl")?
            .stdout
            .context("Could not capture standard output")?;
        debug!("Calling {:?}", &cmd);

        let reader = BufReader::new(stdout);

        reader
            .lines()
            .filter_map(Result::ok)
            .for_each(|line| println!("{}", line));

        Ok(())
    }

    pub fn compile(self, srcs: &Vec<PathBuf>, dst: &PathBuf) -> Result<(), anyhow::Error> {
        let paths: Vec<&str> = srcs.iter().map(|src| src.to_str().unwrap()).collect();

        let mut cmd = Command::new("erlc");
        let cmd = cmd
            .args(&["-b", "beam"])
            .args(&["-o", dst.parent().unwrap().to_str().unwrap()])
            .args(paths.as_slice());
        debug!("Calling {:?}", &cmd);
        cmd.output().map(|_| ()).context("Error running erlc")
    }
}
