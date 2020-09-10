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
    erl: PathBuf,
    erlc: PathBuf,
}

impl Default for Toolchain {
    fn default() -> Toolchain {
        let archive = Archive::default()
            .with_url("https://github.com/erlang/otp/archive/OTP-23.0.3.tar.gz".to_string())
            .with_sha1("b8ec65b335d8a98318870fb2250fc4409e34975a".to_string())
            .with_prefix("otp-OTP-23.0.3".to_string());

        let erl = PathBuf::from("erl");
        let erlc = PathBuf::from("erlc");

        let root = PathBuf::from(".");

        Toolchain {
            archive,
            erl,
            erlc,
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

        let erl = root.join("bin").join("erl");
        let erlc = root.join("bin").join("erlc");

        Toolchain {
            root,
            erl,
            erlc,
            ..self.clone()
        }
    }

    pub fn name(&self) -> String {
        "erlang".to_string()
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
            .args(args.as_slice())
            .spawn()
            .context("Could not spawn erl")?
            .stdout
            .context("Could not capture standard output/error")?;
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

        let mut cmd = Command::new("erlc");
        let cmd = cmd
            // TODO(@ostera): consider enabling `-server`
            .args(&["-server", "-b", "beam", "-Wall", "-Werror"])
            .args(includes.as_slice())
            .args(extra_libs.as_slice())
            .args(&["-o", dst.parent().unwrap().to_str().unwrap()])
            .args(&["--"])
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

impl IntoToolchainBuilder for Toolchain {
    fn toolchain_builder(&self) -> ToolchainBuilder {
        let root = self.root.clone();

        let erlc = self.erlc.clone();
        let is_cached = Box::new(move || {
            Command::new(erlc.clone())
                .output()
                .context("Could not call erlc")
                .map(|output| output.status.success())
        });

        let build_toolchain = Box::new(move || {
            let root = root.clone();
            debug!("Starting to build Erlang toolchain at: {:?}", root);
            let otp_build = std::fs::canonicalize(root.join("otp_build"))
                .context("Could not find otp_build!")?;
            let otp_build = Command::new(otp_build)
                .args(&["autoconf"])
                .current_dir(&root)
                .output()
                .context("Could not spawn otp_build")?;

            if otp_build.status.success() {
                Ok(())
            } else {
                std::io::stdout().write_all(&otp_build.stdout).unwrap();
                std::io::stderr().write_all(&otp_build.stderr).unwrap();
                Err(anyhow!("Error running otp_build"))
            }?;

            let configure = std::fs::canonicalize(root.join("configure"))
                .context("Could not find configure!")?;
            let configure = Command::new(configure)
                .current_dir(&root)
                .output()
                .context("Could not spawn configure")?;

            if configure.status.success() {
                Ok(())
            } else {
                std::io::stdout().write_all(&configure.stdout).unwrap();
                std::io::stderr().write_all(&configure.stderr).unwrap();
                Err(anyhow!("Error running configure"))
            }?;

            debug!("Erlang: running make -j32 in {:?}", &root);
            let make = Command::new("make")
                .stdout(Stdio::piped())
                .args(&["-j32"])
                .current_dir(&root)
                .output()
                .context("Could not spawn make")?;
            debug!("Erlang: {:?}", &make.status.success());

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
