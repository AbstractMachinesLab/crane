use crate::build::{Artifact, BuildContext, BuildRule, Rule};
use crate::label::Label;
use anyhow::{anyhow, Context};
use glob::glob;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::path::PathBuf;
use toml::Value;

#[derive(Debug, Clone)]
pub struct ClojureLibrary {
    name: Label,
    sources: Vec<PathBuf>,
    dependencies: Vec<Label>,
    outputs: Vec<PathBuf>,
}

impl ClojureLibrary {
    pub fn set_sources(&self, sources: Vec<PathBuf>) -> ClojureLibrary {
        ClojureLibrary {
            sources,
            ..self.clone()
        }
    }

    pub fn sources(&self) -> Vec<PathBuf> {
        self.sources.clone()
    }
}

impl Rule for ClojureLibrary {
    fn as_rule(self) -> BuildRule {
        BuildRule::ClojureLibrary(self)
    }

    fn new(name: Label) -> ClojureLibrary {
        ClojureLibrary {
            name,
            sources: vec![],
            dependencies: vec![],
            outputs: vec![],
        }
    }

    fn set_name(&self, name: Label) -> ClojureLibrary {
        ClojureLibrary {
            name,
            ..self.clone()
        }
    }

    fn set_dependencies(&self, dependencies: Vec<Label>) -> ClojureLibrary {
        ClojureLibrary {
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

    fn inputs(&self, _ctx: &BuildContext) -> Vec<PathBuf> {
        self.sources.clone()
    }

    fn outputs(&self, ctx: &BuildContext) -> Vec<Artifact> {
        vec![Artifact {
            inputs: self.inputs(&ctx),
            outputs: self
                .sources
                .iter()
                .map(|file| PathBuf::from("ebin").join(file).with_extension("beam"))
                .collect(),
        }]
    }

    fn build(&mut self, ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        let transitive_beam_files: HashSet<PathBuf> = ctx
            .transitive_dependencies(&self.clone().as_rule())
            .iter()
            .flat_map(|dep| dep.outputs(&ctx))
            .flat_map(|artifact| artifact.outputs)
            .collect();

        let beam_files: Vec<PathBuf> = vec![]
            .iter()
            .chain(transitive_beam_files.iter())
            .cloned()
            .collect();

        ctx.toolchain()
            .clojerl()
            .compile(&self.sources, &beam_files)
    }
}

impl TryFrom<(toml::Value, &PathBuf)> for ClojureLibrary {
    type Error = anyhow::Error;

    fn try_from(input: (toml::Value, &PathBuf)) -> Result<ClojureLibrary, anyhow::Error> {
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
            .unwrap_or(&Value::Array(vec![Value::String("*.clje".to_string())]))
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

        Ok(ClojureLibrary::new(name)
            .set_sources(sources)
            .set_dependencies(dependencies))
    }
}
