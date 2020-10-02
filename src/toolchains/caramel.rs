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
    caramelc: PathBuf,
}

impl Default for Toolchain {
    fn default() -> Toolchain {
        let archive = Archive::default()
            .as_release()
            .with_url(
                "https://github.com/AbstractMachinesLab/caramel/releases/tag/v0.0.4".to_string(),
            )
            .with_sha1("eb14a3d2ec697d42a6ea7cf565d79141846aa538".to_string())
            .with_prefix("".to_string());

        let caramelc = PathBuf::from("caramelc.exe");
        let root = PathBuf::from(".");

        Toolchain {
            archive,
            caramelc,
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
        Toolchain { archive, ..self }
    }

    pub fn with_root(self, root: PathBuf) -> Toolchain {
        let root = root
            .join(self.name())
            .join(self.archive().hash())
            .join(self.archive().prefix());

        let caramelc = root.join("caramelc.exe");

        Toolchain {
            root,
            caramelc,
            ..self
        }
    }

    pub fn name(&self) -> String {
        "caramel".to_string()
    }

    pub fn shell(self, code_paths: &[PathBuf]) -> Result<(), anyhow::Error> {
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
        caramel_srcs: &[PathBuf],
        erl_srcs: &[PathBuf],
        includes: &[PathBuf],
        extra_libs: &[PathBuf],
        dst: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        /*
         * Compile OCaml to Erlang
         */
        let caramel_srcs: Vec<&str> = caramel_srcs
            .iter()
            .map(|src| src.to_str().unwrap())
            .collect();

        let mut cmd = Command::new(self.caramelc);
        let cmd = cmd
            .args(&["compile"])
            .args(&["--output-dir", dst.parent().unwrap().to_str().unwrap()])
            .args(caramel_srcs.as_slice());

        debug!("Calling {:#?}", &cmd);
        let output = cmd.output()?;
        if output.status.success() {
            Ok(())
        } else {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            Err(anyhow!("Error running erlc"))
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
            .args(&["-b", "beam", "-Wall", "-Werror"])
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
        let caramelc = self.caramelc.clone();
        let is_cached = Box::new(move || Ok(std::fs::metadata(caramelc.clone()).is_ok()));

        let build_toolchain = Box::new(move || {
            debug!("Nothing to do here!");
            Ok(())
        });

        ToolchainBuilder {
            name: self.name(),
            archive: self.archive().clone(),
            is_cached,
            build_toolchain,
        }
    }
}
