use anyhow::{anyhow, Context};
use log::debug;
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
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
            language: "clojerl".to_string(),
            version: "0.7.0".to_string(),
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

        let mut cmd = Command::new(
            "~/.crane/toolchains/clojerl/0fc57cbf85110ae59429d429f98c34d781a4fb71/bin/clojerl",
        );
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

    pub fn compile(
        self,
        srcs: &Vec<PathBuf>,
        extra_libs: &Vec<PathBuf>,
        dst: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        let paths: Vec<&str> = srcs.iter().map(|src| src.to_str().unwrap()).collect();

        let extra_lib_paths: HashSet<&str> = extra_libs
            .iter()
            .map(|beam| beam.parent().unwrap().to_str().unwrap())
            .collect();
        let extra_libs: Vec<&str> = extra_lib_paths
            .iter()
            .cloned()
            .flat_map(|dir| vec!["-pa", dir])
            .collect();

        let mut cmd = Command::new(
            "/home/ostera/.crane/toolchains/clojerl/0fc57cbf85110ae59429d429f98c34d781a4fb71/bin/clojerl",
        );
        let cmd = cmd
            .args(extra_libs.as_slice())
            .args(&["-o", dst.parent().unwrap().to_str().unwrap()])
            .args(&["--time", "--compile"])
            .args(paths.as_slice());

        debug!("Calling {:#?}", &cmd);
        let output = cmd.output()?;

        if output.status.success() {
            Ok(())
        } else {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            Err(anyhow!("Error running erlc"))
        }
    }
}
