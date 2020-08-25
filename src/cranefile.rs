use crate::build_rules::build_rule::BuildRule;
use crate::build_rules::library::Library;
use crate::build_rules::shell::Shell;
use crate::build_rules::test::Test;
use crate::model::target::Label;
use anyhow::{anyhow, Context, Error};
use glob::glob;
use log::{error, info};
use std::fs;
use std::path::PathBuf;
use std::vec::Vec;
use toml::Value;

pub const CRANEFILE: &str = "CRANE";

#[derive(Debug, Clone)]
pub struct Cranefile {
    path: PathBuf,
    contents: toml::Value,
    rules: Vec<BuildRule>,
}

impl Default for Cranefile {
    fn default() -> Self {
        Cranefile {
            path: PathBuf::from("./crane"),
            contents: "".parse::<Value>().unwrap(),
            rules: vec![],
        }
    }
}

impl Cranefile {
    pub fn from_file(path: PathBuf) -> Result<Cranefile, Error> {
        let contents = fs::read_to_string(&path)?.parse::<Value>()?;

        let parent = &path.parent().unwrap().to_path_buf();

        let shells = &contents
            .get("shell")
            .unwrap_or(&Value::Array(vec![]))
            .clone();
        let shells = parse_shells(&parent, shells)?;

        let libraries = &contents
            .get("library")
            .unwrap_or(&Value::Array(vec![]))
            .clone();
        let libraries = parse_libraries(&parent, libraries)?;

        let tests = &contents
            .get("test")
            .unwrap_or(&Value::Array(vec![]))
            .clone();
        let tests = parse_tests(&parent, tests)?;

        let rules: Vec<BuildRule> = vec![libraries, tests, shells]
            .iter()
            .flatten()
            .cloned()
            .collect();

        Ok(Cranefile {
            path,
            contents,
            rules,
        })
    }

    pub fn rules(self) -> Vec<BuildRule> {
        self.rules
    }
}

fn parse_libraries(path: &PathBuf, libs: &toml::Value) -> Result<Vec<BuildRule>, anyhow::Error> {
    match libs {
        Value::Array(libs) => libs
            .iter()
            .map(|lib| {
                let name = lib
                    .get("name")
                    .context("Rule does not have a valid name")?
                    .as_str()
                    .context("Names should always be strings")?
                    .to_string();
                let name: Label = name.into();
                let name = name.canonicalize(&path);

                let files = match &lib
                    .get("files")
                    .unwrap_or(&Value::Array(vec![Value::String("*.erl".to_string())]))
                {
                    Value::Array(files) => files
                        .iter()
                        .flat_map(|f| match f {
                            Value::String(name) => glob(path.join(name).to_str().unwrap())
                                .expect("Could not read glob")
                                .filter_map(Result::ok)
                                .collect(),
                            _ => vec![],
                        })
                        .collect(),
                    _ => vec![],
                };

                let dependencies = match &lib.get("deps").unwrap_or(&Value::Array(vec![])) {
                    Value::Array(deps) => deps
                        .iter()
                        .map(|x| {
                            let label: Label = x.to_string().into();
                            Ok(label.canonicalize(&path))
                        })
                        .collect(),
                    e => Err(anyhow!(
                        "We expected an array of dependencies, but instead we got: {:?}",
                        e
                    )),
                }?;

                let lib = Library::new(name)
                    .set_files(files)
                    .set_dependencies(dependencies);

                Ok(BuildRule::Library(lib))
            })
            .collect(),
        x => Err(anyhow!(
            "We expected an array of libraries, but instead got: {:?}",
            x
        )),
    }
}

fn parse_tests(path: &PathBuf, libs: &toml::Value) -> Result<Vec<BuildRule>, anyhow::Error> {
    match libs {
        Value::Array(libs) => libs
            .iter()
            .map(|lib| {
                let name = lib
                    .get("name")
                    .context("Rule does not have a valid name")?
                    .as_str()
                    .context("Names should always be strings")?
                    .to_string();
                let name: Label = name.into();
                let name = name.canonicalize(&path);

                if !name.ends_with("_test") {
                    return Err(anyhow!("Test names should always end with _test"));
                }

                let file = lib
                    .get("file")
                    .context("Test rule does not have a file")?
                    .as_str()
                    .context("Test rule should have a single file as a string")?
                    .to_string();

                let dependencies = match &lib.get("deps").unwrap_or(&Value::Array(vec![])) {
                    Value::Array(deps) => deps
                        .iter()
                        .map(|x| {
                            let label: Label = x.to_string().into();
                            Ok(label.canonicalize(&path))
                        })
                        .collect(),
                    e => Err(anyhow!(
                        "We expected an array of dependencies, but instead we got: {:?}",
                        e
                    )),
                }?;

                Ok(BuildRule::Test(Test {
                    name,
                    file,
                    dependencies,
                }))
            })
            .collect(),
        x => Err(anyhow!(
            "We expected an array of tests, but instead got: {:?}",
            x
        )),
    }
}

fn parse_shells(path: &PathBuf, libs: &toml::Value) -> Result<Vec<BuildRule>, anyhow::Error> {
    match libs {
        Value::Array(libs) => libs
            .iter()
            .map(|lib| {
                let name = lib
                    .get("name")
                    .context("Rule does not have a valid name")?
                    .as_str()
                    .context("Names should always be strings")?
                    .to_string();
                let name: Label = name.into();
                let name = name.canonicalize(&path);

                let dependencies = match &lib.get("deps").unwrap_or(&Value::Array(vec![])) {
                    Value::Array(deps) => deps
                        .iter()
                        .map(|x| {
                            let label: Label = x.to_string().into();
                            Ok(label.canonicalize(&path))
                        })
                        .collect(),
                    e => Err(anyhow!(
                        "We expected an array of dependencies, but instead we got: {:?}",
                        e
                    )),
                }?;

                Ok(BuildRule::Shell(
                    Shell::new(name).set_dependencies(dependencies),
                ))
            })
            .collect(),
        x => Err(anyhow!(
            "We expected an array of tests, but instead got: {:?}",
            x
        )),
    }
}
