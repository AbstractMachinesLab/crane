use super::archive::Archive;
use super::IntoToolchainBuilder;
use super::ToolchainBuilder;
use anyhow::{anyhow, Context};
use log::debug;
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct Toolchain {
    archive: Archive,
    root: PathBuf,
    gleamc: PathBuf,
}

impl Default for Toolchain {
    fn default() -> Toolchain {
        let archive = Archive::default()
            .with_url("https://github.com/ostera/gleam/archive/gleamc.tar.gz".to_string())
            .with_sha1("13cc365903efb7fa16b94f09da9eb1f58285f1aa".to_string())
            .with_prefix("gleam-gleamc".to_string());

        let gleamc = PathBuf::from("gleamc");
        let root = PathBuf::from(".");

        Toolchain {
            archive,
            gleamc,
            root,
        }
    }
}

impl Toolchain {
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn archive(&self) -> &Archive {
        &self.archive
    }

    pub fn with_archive(self, archive: Archive) -> Toolchain {
        Toolchain {
            archive,
            ..self.clone()
        }
    }

    pub fn with_root(self, root: PathBuf) -> Toolchain {
        let root = root
            .join(self.name())
            .join(self.archive().sha1())
            .join(self.archive().prefix());

        let gleamc = root.join("target").join("release").join("gleam");

        Toolchain {
            root,
            gleamc,
            ..self.clone()
        }
    }

    pub fn name(&self) -> String {
        "gleam".to_string()
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

    pub fn compile(
        self,
        gleam_srcs: &Vec<PathBuf>,
        erl_srcs: &Vec<PathBuf>,
        includes: &Vec<PathBuf>,
        extra_libs: &Vec<PathBuf>,
        dst: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        /*
         * Compile Gleam to Erlang
         */
        let gleam_srcs: Vec<&str> = gleam_srcs.iter().map(|src| src.to_str().unwrap()).collect();

        let mut cmd = Command::new(self.gleamc);
        let cmd = cmd
            .args(&["compile"])
            .args(&["--output-dir", dst.parent().unwrap().to_str().unwrap()])
            .args(gleam_srcs.as_slice());

        debug!("Calling {:#?}", &cmd);
        let output = {
            let output = cmd.output()?;
            if output.status.success() {
                Ok(())
            } else {
                std::io::stdout().write_all(&output.stdout).unwrap();
                std::io::stderr().write_all(&output.stderr).unwrap();
                Err(anyhow!("Error running erlc"))
            }
        }?;

        /*
         * Compile Erlang to BEAM now
         */

        let erl_paths: Vec<&str> = erl_srcs.iter().map(|src| src.to_str().unwrap()).collect();

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

        let mut cmd = Command::new("erlc");
        let cmd = cmd
            // TODO(@ostera): consider enabling `-server`
            .args(&["-server", "-b", "beam", "-Wall", "-Werror"])
            .args(includes.as_slice())
            .args(extra_libs.as_slice())
            .args(&["-o", dst.parent().unwrap().to_str().unwrap()])
            .args(&["--"])
            .args(erl_paths.as_slice());

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

impl IntoToolchainBuilder for Toolchain {
    fn toolchain_builder(&self) -> ToolchainBuilder {
        let root = self.root.clone();

        let gleamc = self.gleamc.clone();
        let is_cached = Box::new(move || {
            debug!("Gleamc: calling {:?}", gleamc);
            Command::new(gleamc.clone())
                .args(&["-V"])
                .output()
                .context("Could not call gleamc")
                .map(|output| output.status.success())
        });

        let build_toolchain = Box::new(move || Err(anyhow!("Can not build gleam just yet")));

        ToolchainBuilder {
            name: self.name(),
            archive: self.archive().clone(),
            is_cached,
            build_toolchain,
        }
    }
}
