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
    gleam_bin: PathBuf,
}

impl Default for Toolchain {
    fn default() -> Toolchain {
        let archive = Archive::default()
            .as_source()
            .with_name("gleam".to_string())
            .with_url("https://github.com/gleam-lang/gleam/archive/main.tar.gz".to_string())
            .with_sha1("26de3f4888c5bd887825d0b82a8feb41fc811ebe".to_string())
            .with_prefix("gleam-main".to_string());

        let gleam_bin = PathBuf::from("gleam");
        let root = PathBuf::from(".");

        Toolchain {
            archive,
            gleam_bin,
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

        let gleam_bin = root.join("target").join("release").join("gleam");

        Toolchain {
            root,
            gleam_bin,
            ..self
        }
    }

    pub fn name(&self) -> String {
        "gleam".to_string()
    }

    pub fn compile(self, gleam_srcs: &[PathBuf]) -> Result<(), anyhow::Error> {
        /*
         * Compile Gleam to Erlang
         */
        let gleam_srcs: Vec<&str> = gleam_srcs.iter().map(|src| src.to_str().unwrap()).collect();

        let mut cmd = Command::new(self.gleam_bin);
        let cmd = cmd.args(&["build", "."]);

        debug!("Calling {:#?}", &cmd);
        let output = cmd.output()?;
        if output.status.success() {
            Ok(())
        } else {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            Err(anyhow!("Error running gleam"))
        }
    }
}

impl IntoToolchainBuilder for Toolchain {
    fn toolchain_builder(&self) -> ToolchainBuilder {
        let root = self.root.clone();

        let gleam_bin = self.gleam_bin.clone();
        let is_cached = Box::new(move || Ok(std::fs::metadata(gleam_bin.clone()).is_ok()));

        let build_toolchain = Box::new(move || {
            let root = root.clone();
            debug!("Cargo: running cargo in {:?}", &root);
            let cargo = Command::new("cargo")
                .stdout(Stdio::piped())
                .args(&["build", "--release"])
                .current_dir(&root)
                .output()
                .context("Could not spawn cargo")?;
            debug!("Cargo: {:?}", &cargo.status.success());

            if cargo.status.success() {
                Ok(())
            } else {
                std::io::stdout().write_all(&cargo.stdout).unwrap();
                std::io::stderr().write_all(&cargo.stderr).unwrap();
                Err(anyhow!("Error running cargo"))
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
