use crate::build::{Artifact, BuildPlan, BuildRule, Rule};
use crate::label::Label;
use crate::toolchains::Toolchains;
use anyhow::{anyhow, Context};
use glob::glob;
use std::convert::TryFrom;
use std::path::PathBuf;
use toml::Value;

#[derive(Debug, Clone)]
pub struct ElixirLibrary {
    name: Label,
    sources: Vec<PathBuf>,
    dependencies: Vec<Label>,
    outputs: Vec<PathBuf>,
}

impl ElixirLibrary {
    pub fn set_sources(&self, sources: Vec<PathBuf>) -> ElixirLibrary {
        ElixirLibrary {
            sources,
            ..self.clone()
        }
    }

    pub fn sources(&self) -> Vec<PathBuf> {
        self.sources.clone()
    }
}

impl Rule for ElixirLibrary {
    fn as_rule(self) -> BuildRule {
        BuildRule::ElixirLibrary(self)
    }

    fn new(name: Label) -> ElixirLibrary {
        ElixirLibrary {
            name,
            sources: vec![],
            dependencies: vec![],
            outputs: vec![],
        }
    }

    fn set_name(&self, name: Label) -> ElixirLibrary {
        ElixirLibrary {
            name,
            ..self.clone()
        }
    }

    fn set_dependencies(&self, dependencies: Vec<Label>) -> ElixirLibrary {
        ElixirLibrary {
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
        self.sources.clone()
    }

    fn set_inputs(self, sources: Vec<PathBuf>) -> ElixirLibrary {
        ElixirLibrary { sources, ..self }
    }

    fn outputs(&self, deps: &[Artifact]) -> Vec<Artifact> {
        vec![Artifact {
            inputs: self.inputs(&deps),
            outputs: self
                .sources
                .iter()
                .map(|file| {
                    file.parent()
                        .unwrap()
                        .join("Elixir")
                        .with_extension(file.file_name().unwrap())
                        .with_extension("beam")
                })
                .collect(),
        }]
    }

    fn build(&mut self, _plan: &BuildPlan, toolchains: &Toolchains) -> Result<(), anyhow::Error> {
        let beam_files: Vec<PathBuf> = vec![];
        let headers: Vec<PathBuf> = vec![];
        toolchains
            .elixir()
            .compile(&self.sources, &headers, &beam_files)
    }
}

impl TryFrom<(toml::Value, &PathBuf)> for ElixirLibrary {
    type Error = anyhow::Error;

    fn try_from(input: (toml::Value, &PathBuf)) -> Result<ElixirLibrary, anyhow::Error> {
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

        let sources = match &lib
            .get("sources")
            .unwrap_or(&Value::Array(vec![Value::String("*.ex".to_string())]))
        {
            Value::Array(sources) => sources
                .iter()
                .flat_map(|f| match f {
                    Value::String(name) => glob(path.join(name).to_str().unwrap())
                        .expect("Could not read glob")
                        .filter_map(Result::ok)
                        .collect(),
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

        Ok(ElixirLibrary::new(name)
            .set_sources(sources)
            .set_dependencies(dependencies))
    }
}
