use crate::cranefile::{Cranefile, CRANEFILE};
use crate::label::Label;
use crate::toolchains::Toolchain;
use anyhow::Context;
use log::debug;
use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;

pub const WORKSPACE: &str = "workspace";

#[derive(Debug, Clone, Default)]
pub struct Workspace {
    name: String,
    toolchains: Toolchain,
    dependencies: Vec<Label>,
    root: PathBuf,
}

impl Workspace {
    pub fn new() -> Result<Workspace, anyhow::Error> {
        let cwd = std::fs::canonicalize(PathBuf::from(&"."))?;
        let workspace_file = Workspace::find_workspace_file_upwards(&cwd)?;
        let parsed_workspace = Workspace::from_toml_file(workspace_file)?;
        Ok(Workspace {
            root: PathBuf::from(&"."),
            ..parsed_workspace
        })
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn toolchains(&self) -> Toolchain {
        self.toolchains.clone()
    }

    pub fn with_toolchains(&self, toolchains: Toolchain) -> Workspace {
        Workspace {
            toolchains,
            ..self.clone()
        }
    }

    pub fn dependencies(&self) -> &Vec<Label> {
        &self.dependencies
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn cranefiles(&self) -> Vec<Cranefile> {
        Workspace::find_files(&self.root)
    }

    pub fn from_toml_file(path: PathBuf) -> Result<Workspace, anyhow::Error> {
        let toml = fs::read_to_string(&path)
            .context(format!("Could not read file {:?}", &path))?
            .parse::<toml::Value>()
            .context(format!("Could not parse file {:?} as TOML", &path))?;

        debug!("Loading workspace file from {:?}", &path);
        Workspace::try_from(toml)
    }

    fn find_workspace_file_upwards(cwd: &PathBuf) -> Result<PathBuf, anyhow::Error> {
        let here = &cwd.join(WORKSPACE);
        debug!("Searching for workspace file in {:?}", here);
        if fs::metadata(here).is_ok() {
            debug!("Found it! Continuing...");
            Ok(here.to_path_buf())
        } else {
            let parent = cwd.parent().context(
                "Reached the top of the file system and could not find a workspace file",
            )?;
            Workspace::find_workspace_file_upwards(&parent.to_path_buf())
        }
    }

    fn find_files(root: &PathBuf) -> Vec<Cranefile> {
        if root.is_dir() {
            fs::read_dir(root)
                .unwrap_or_else(|err| {
                    panic!("Could not read directory: {:?} due to {:?}", root, err)
                })
                .flat_map(|entry| {
                    let entry = entry.unwrap_or_else(|_| panic!("Could not read entry"));
                    let path = entry.path();

                    if path.is_dir() {
                        Workspace::find_files(&path)
                    } else {
                        let name = path.file_name().unwrap().to_str().unwrap();
                        if name.eq_ignore_ascii_case(CRANEFILE) {
                            let cranefile =
                                Cranefile::from_file(path.clone()).unwrap_or_else(|_| {
                                    panic!("Could not read Cranefile at: {:?}", path)
                                });
                            vec![cranefile]
                        } else {
                            vec![]
                        }
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    pub fn clean(self) -> Result<(), anyhow::Error> {
        std::fs::remove_dir_all(self.root().join(".crane"))
            .context("Could not remove .crane folder")
    }
}

impl TryFrom<toml::Value> for Workspace {
    type Error = anyhow::Error;

    fn try_from(toml: toml::Value) -> Result<Workspace, anyhow::Error> {
        let workspace = toml
            .get("workspace")
            .context("Workspace file must have a workspace section")?;
        let name = workspace
            .get("name")
            .context("Workspace must have a name field")?
            .as_str()
            .context("Workspace name field must be a string")?
            .to_string();

        let toolchains = if let Some(toolchains) = toml.get("toolchains") {
            Toolchain::try_from(toolchains.clone())
        } else {
            Ok(Toolchain::default())
        }?;

        debug!("Found Toolchains: {:?}", toolchains);

        Ok(Workspace {
            toolchains,
            name,
            ..Workspace::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demans_workspace_name() {
        let toml: toml::Value = r#"
[workspace]
[toolchains]
[dependencies]
        "#
        .parse::<toml::Value>()
        .unwrap();
        let workspace = Workspace::try_from(toml);
        assert_eq!(true, workspace.is_err());
    }

    #[test]
    fn parses_toml_into_workspace_struct() {
        let toml: toml::Value = r#"
[workspace]
name = "tiny_lib"

[toolchains]

[dependencies]
        "#
        .parse::<toml::Value>()
        .unwrap();
        let workspace = Workspace::try_from(toml).unwrap();
        assert_eq!(workspace.name(), "tiny_lib");
    }

    #[test]
    fn uses_defaults() {
        let toml: toml::Value = r#"
[workspace]
name = "tiny_lib"

[toolchains]

[dependencies]
        "#
        .parse::<toml::Value>()
        .unwrap();
        let workspace = Workspace::try_from(toml).unwrap();
        assert_eq!(
            workspace.toolchains().erlang().archive().url(),
            toolchains::erlang::Toolchain::default().archive().url()
        );
        assert_eq!(
            workspace.toolchains().erlang().archive().sha1(),
            toolchains::erlang::Toolchain::default().archive().sha1()
        );
        assert_eq!(
            workspace.toolchains().erlang().archive().prefix(),
            toolchains::erlang::Toolchain::default().archive().prefix()
        );
        assert_eq!(workspace.dependencies().len(), 0);
    }

    #[test]
    fn allows_for_custom_toolchains() {
        let toml: toml::Value = r#"
[workspace]
name = "tiny_lib"

[toolchains]
erlang = { archive_url = "master", prefix = "otp-master" }
gleam = { archive_url = "https://github.com/forked/gleam", sha1 = "test" }

[dependencies]
        "#
        .parse::<toml::Value>()
        .unwrap();
        let workspace = Workspace::try_from(toml).unwrap();
        assert_eq!(workspace.toolchains().erlang().archive().url(), "master");
        assert_eq!(
            workspace.toolchains().erlang().archive().prefix(),
            "otp-master"
        );
        assert_eq!(
            workspace.toolchains().gleam().archive().url(),
            "https://github.com/forked/gleam"
        );
        assert_eq!(workspace.toolchains().gleam().archive().sha1(), "test");
        assert_eq!(workspace.dependencies().len(), 0);
    }
}
