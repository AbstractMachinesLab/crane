use crate::cranefile::{Cranefile, CRANEFILE};
use crate::label::Label;
use crate::toolchains::Toolchain;
use anyhow::Context;
use log::debug;
use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;
use toml;

pub const WORKSPACE: &str = "workspace";

#[derive(Debug, Clone, Default)]
pub struct Workspace {
    name: String,
    toolchain: Toolchain,
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

    pub fn toolchain(&self) -> &Toolchain {
        &self.toolchain
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
        if let Ok(_) = fs::metadata(here) {
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
                    let entry = entry.expect(&format!("Could not read entry"));
                    let path = entry.path();

                    if path.is_dir() {
                        Workspace::find_files(&path)
                    } else {
                        let name = path.file_name().unwrap().to_str().unwrap();
                        if name.eq_ignore_ascii_case(CRANEFILE) {
                            let cranefile = Cranefile::from_file(path.clone())
                                .expect(&format!("Could not read Cranefile at: {:?}", path));
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

        Ok(Workspace {
            name,
            ..Workspace::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::model::workspace::*;
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
        assert_eq!(workspace.toolchain().language(), "erlang");
        assert_eq!(workspace.toolchain().version(), "23");
        assert_eq!(workspace.dependencies().len(), 0);
    }
}
