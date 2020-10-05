use super::archive::Archive;
use super::IntoToolchainBuilder;
use super::ToolchainBuilder;
use anyhow::anyhow;
use log::debug;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Toolchain {
    archive: Archive,
    root: PathBuf,
    lumen: PathBuf,
}

impl Default for Toolchain {
    fn default() -> Toolchain {
        let archive = Archive::default()
            .as_release()
            .with_name("lumen-0.1.0-nightly".to_string())
            .with_tag("0.1.0-nightly-2020-09-02".to_string())
            .with_url("https://github.com/lumen/lumen".to_string())
            .with_sha1("53e0ca166c0b5106c997f56cea3045a3b110e255".to_string())
            .with_prefix("lumen".to_string());

        let lumen = PathBuf::from("bin/lumen");
        let root = PathBuf::from(".");

        Toolchain {
            archive,
            lumen,
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

        let lumen = root.join("bin/lumen");

        Toolchain {
            root,
            lumen,
            ..self
        }
    }

    pub fn name(&self) -> String {
        "lumen".to_string()
    }

    pub fn compile(self, caramel_srcs: &[PathBuf]) -> Result<(), anyhow::Error> {
        let caramel_srcs: Vec<&str> = caramel_srcs
            .iter()
            .map(|src| src.to_str().unwrap())
            .collect();

        let mut cmd = Command::new(self.lumen);
        let cmd = cmd.args(&["compile"]).args(caramel_srcs.as_slice());

        debug!("Calling {:?}", &cmd);
        let output = cmd.output()?;
        if output.status.success() {
            Ok(())
        } else {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            Err(anyhow!("Error running lumen"))
        }
    }
}

impl IntoToolchainBuilder for Toolchain {
    fn toolchain_builder(&self) -> ToolchainBuilder {
        let lumen = self.lumen.clone();
        let is_cached = Box::new(move || Ok(std::fs::metadata(lumen.clone()).is_ok()));

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
