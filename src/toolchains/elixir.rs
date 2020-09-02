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
            language: "elixir".to_string(),
            version: "1.10.0".to_string(),
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

        let mut cmd = Command::new("/home/ostera/.crane/toolchains/elixir/1.10.4-1145dc01680aab7094f8a6dbd38b65185e14adb4/bin/iex");
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
        includes: &Vec<PathBuf>,
        extra_libs: &Vec<PathBuf>,
        dst: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        let paths: Vec<&str> = srcs.iter().map(|src| src.to_str().unwrap()).collect();

        let include_paths: HashSet<&str> = includes
            .iter()
            .map(|hrl| hrl.parent().unwrap().to_str().unwrap())
            .collect();
        let includes: Vec<&str> = include_paths
            .iter()
            .cloned()
            .flat_map(|dir| vec!["-I", dir])
            .collect();

        let extra_lib_paths: HashSet<&str> = extra_libs
            .iter()
            .map(|beam| beam.parent().unwrap().to_str().unwrap())
            .collect();
        let extra_libs: Vec<&str> = extra_lib_paths
            .iter()
            .cloned()
            .flat_map(|dir| vec!["-pa", dir])
            .collect();

        let mut cmd = Command::new("/home/ostera/.crane/toolchains/elixir/1.10.4-1145dc01680aab7094f8a6dbd38b65185e14adb4/bin/elixirc");
        let cmd = cmd
            // TODO(@ostera): consider passing -I and -pa's into ERL_COMPILER_OPTIONS
            .args(&["--warnings-as-errors"])
            .args(&["-o", dst.parent().unwrap().to_str().unwrap()])
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
