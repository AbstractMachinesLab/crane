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
            .with_name("caramel".to_string())
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

    pub fn compile(self, caramel_srcs: &[PathBuf]) -> Result<(), anyhow::Error> {
        let caramel_srcs: Vec<&str> = caramel_srcs
            .iter()
            .map(|src| src.to_str().unwrap())
            .collect();

        let mut cmd = Command::new(self.caramelc);
        let cmd = cmd.args(&["compile"]).args(caramel_srcs.as_slice());

        debug!("Calling {:#?}", &cmd);
        let output = cmd.output()?;
        if output.status.success() {
            Ok(())
        } else {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            Err(anyhow!("Error running caramelc"))
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
