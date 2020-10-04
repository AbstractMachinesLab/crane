use crate::build::{Artifact, BuildPlan, BuildRule, Rule};
use crate::label::Label;
use crate::toolchains::Toolchains;
use anyhow::{anyhow, Context};
use glob::glob;
use std::convert::TryFrom;
use std::path::PathBuf;
use toml::Value;

#[derive(Debug, Clone)]
pub struct GleamLibrary {
    name: Label,
    config: PathBuf,
    sources: Vec<PathBuf>,
    dependencies: Vec<Label>,
    outputs: Vec<PathBuf>,
}

impl GleamLibrary {
    pub fn set_config(self, config: PathBuf) -> GleamLibrary {
        GleamLibrary { config, ..self }
    }

    pub fn set_sources(self, sources: Vec<PathBuf>) -> GleamLibrary {
        GleamLibrary { sources, ..self }
    }

    pub fn sources(&self) -> Vec<PathBuf> {
        self.sources.clone()
    }
}

impl Rule for GleamLibrary {
    fn as_rule(self) -> BuildRule {
        BuildRule::GleamLibrary(self)
    }

    fn new(name: Label) -> GleamLibrary {
        GleamLibrary {
            name,
            config: PathBuf::from("gleam.toml"),
            sources: vec![],
            dependencies: vec![],
            outputs: vec![],
        }
    }

    fn set_name(&self, name: Label) -> GleamLibrary {
        GleamLibrary {
            name,
            ..self.clone()
        }
    }

    fn set_dependencies(&self, dependencies: Vec<Label>) -> GleamLibrary {
        GleamLibrary {
            dependencies,
            ..self.clone()
        }
    }

    fn name(&self) -> Label {
        self.name.clone()
    }
    fn dependencies(&self) -> Vec<Label> {
        self.dependencies.clone()
    }

    fn inputs(&self, _deps: &[Artifact]) -> Vec<PathBuf> {
        let mut inputs = self.sources.clone();
        inputs.push(self.config.clone());
        inputs
    }

    fn outputs(&self, deps: &[Artifact]) -> Vec<Artifact> {
        vec![Artifact {
            inputs: self.inputs(&deps),
            outputs: self
                .sources
                .iter()
                .map(|file| file.with_extension("erl"))
                .collect(),
        }]
    }

    fn set_inputs(self, sources: Vec<PathBuf>) -> GleamLibrary {
        GleamLibrary { sources, ..self }
    }

    fn build(&mut self, _plan: &BuildPlan, toolchains: &Toolchains) -> Result<(), anyhow::Error> {
        toolchains.gleam().compile(&self.sources)
    }
}

impl TryFrom<(toml::Value, &PathBuf)> for GleamLibrary {
    type Error = anyhow::Error;

    fn try_from(input: (toml::Value, &PathBuf)) -> Result<GleamLibrary, anyhow::Error> {
        let (lib, path) = input;
        let path = path
            .strip_prefix(PathBuf::from("."))
            .context(format!("Could not strip prefix . from path: {:?}", &path))?
            .to_path_buf();
        let name = lib
            .get("name")
            .context("Rule does not have a valid name")?
            .as_str()
            .context("Names should always be strings")?
            .to_string();
        let name: Label = name.into();
        let name = name.canonicalize(&path);

        let config = match &lib
            .get("config")
            .unwrap_or(&Value::String("gleam.toml".to_string()))
        {
            Value::String(config) => PathBuf::from(config),
            _ => panic!("Gleam library config field must be an string"),
        };

        let sources = match &lib
            .get("srcs")
            .unwrap_or(&Value::Array(vec![Value::String("*.gleam".to_string())]))
        {
            Value::Array(sources) => sources
                .iter()
                .flat_map(|f| match f {
                    Value::String(name) => {
                        let glob_pattern = path.join(name);
                        glob(glob_pattern.to_str().unwrap())
                            .expect(&format!("Could not read glob {:?}", &glob_pattern))
                            .filter_map(Result::ok)
                            .collect()
                    }
                    _ => vec![],
                })
                .map(|file| {
                    file.strip_prefix(&path)
                        .expect(&format!(
                            "Could not strip prefix {:?} from path {:?}",
                            &file, &path
                        ))
                        .to_path_buf()
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

        Ok(GleamLibrary::new(name)
            .set_config(config)
            .set_sources(sources)
            .set_dependencies(dependencies))
    }
}
