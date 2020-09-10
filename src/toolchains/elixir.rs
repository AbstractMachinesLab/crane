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
    elixirc: PathBuf,
}

impl Default for Toolchain {
    fn default() -> Toolchain {
        let archive = Archive::default()
            .with_url("https://github.com/elixir-lang/elixir/archive/v1.10.4.tar.gz".to_string())
            .with_sha1("d8634700f61c72c0e97f1a212919803a86016d2a".to_string())
            .with_prefix("elixir-1.10.4".to_string());

        let elixirc = PathBuf::from("elixirc");
        let root = PathBuf::from(".");

        Toolchain {
            archive,
            elixirc,
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
            .join(self.archive().hash())
            .join(self.archive().prefix());

        let elixirc = root.join("bin").join("elixirc");

        Toolchain {
            root,
            elixirc,
            ..self.clone()
        }
    }

    pub fn name(&self) -> String {
        "elixir".to_string()
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

        let mut cmd = Command::new(self.elixirc.clone());
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
            Err(anyhow!("Error running elixirc"))
        }
    }
}

impl IntoToolchainBuilder for Toolchain {
    fn toolchain_builder(&self) -> ToolchainBuilder {
        let root = self.root.clone();

        let elixirc = self.elixirc.clone();
        let is_cached = Box::new(move || Ok(std::fs::metadata(elixirc.clone()).is_ok()));

        let build_toolchain = Box::new(move || {
            let root = root.clone();
            debug!("Elixir: running make -j32 in {:?}", &root);
            let make = Command::new("make")
                .stdout(Stdio::piped())
                .args(&["-j32"])
                .current_dir(&root)
                .output()
                .context("Could not spawn make")?;
            debug!("Elixir: {:?}", &make.status.success());

            if make.status.success() {
                Ok(())
            } else {
                std::io::stdout().write_all(&make.stdout).unwrap();
                std::io::stderr().write_all(&make.stderr).unwrap();
                Err(anyhow!("Error running make"))
            }
        });

        ToolchainBuilder {
            name: self.name(),
            archive: self.archive().clone(),
            is_cached,
            build_toolchain,
        }
    }
}
