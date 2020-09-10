use crate::build::{Artifact, BuildContext, BuildRule, Rule};
use crate::label::Label;
use anyhow::{anyhow, Context};
use glob::glob;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::path::PathBuf;
use toml::Value;

#[derive(Debug, Clone)]
pub struct GleamLibrary {
    name: Label,
    sources: Vec<PathBuf>,
    dependencies: Vec<Label>,
    outputs: Vec<PathBuf>,
}

impl GleamLibrary {
    pub fn set_sources(&self, sources: Vec<PathBuf>) -> GleamLibrary {
        GleamLibrary {
            sources,
            ..self.clone()
        }
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

    fn inputs(&self, _ctx: &BuildContext) -> Vec<PathBuf> {
        vec![self.sources.clone()]
            .iter()
            .flatten()
            .cloned()
            .collect()
    }

    fn outputs(&self, ctx: &BuildContext) -> Vec<Artifact> {
        vec![Artifact {
            inputs: self.inputs(&ctx),
            outputs: self
                .sources
                .iter()
                .map(|file| file.with_extension("beam"))
                .collect(),
        }]
    }

    fn build(&mut self, ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        let transitive_deps = ctx.transitive_dependencies(&self.clone().as_rule());

        let transitive_headers: HashSet<PathBuf> = transitive_deps
            .iter()
            .flat_map(|dep| dep.outputs(&ctx))
            .flat_map(|artifact| artifact.inputs)
            .map(|path| ctx.output_path().join(path))
            .collect();
        let transitive_headers: Vec<PathBuf> = transitive_headers.iter().cloned().collect();

        if !self.sources.is_empty() {
            let transitive_beam_files: HashSet<PathBuf> = transitive_deps
                .iter()
                .flat_map(|dep| dep.outputs(&ctx))
                .flat_map(|artifact| artifact.outputs)
                .map(|path| ctx.output_path().join(path))
                .collect();

            let beam_files: Vec<PathBuf> = self
                .sources
                .iter()
                .cloned()
                .map(|file| {
                    let file = file
                        .parent()
                        .unwrap()
                        .join("Gleam")
                        .with_extension(file.file_name().unwrap())
                        .with_extension("beam");
                    ctx.declare_output(file)
                })
                .collect();

            let beam_files: Vec<PathBuf> = beam_files
                .iter()
                .chain(transitive_beam_files.iter())
                .cloned()
                .collect();

            let dest = beam_files[0].clone();

            let erlang_sources: Vec<PathBuf> = self
                .sources
                .clone()
                .iter()
                .map(|file| {
                    let file = file
                        .parent()
                        .unwrap()
                        .join(file.file_name().unwrap())
                        .with_extension("erl");
                    ctx.declare_output(file)
                })
                .collect();

            ctx.toolchain().gleam().compile(
                &self.sources,
                &erlang_sources,
                &transitive_headers,
                &beam_files,
                &dest,
            )
        } else {
            Ok(())
        }
    }
}

impl TryFrom<(toml::Value, &PathBuf)> for GleamLibrary {
    type Error = anyhow::Error;

    fn try_from(input: (toml::Value, &PathBuf)) -> Result<GleamLibrary, anyhow::Error> {
        let (lib, path) = input;
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
            .unwrap_or(&Value::Array(vec![Value::String("*.gleam".to_string())]))
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
            .set_sources(sources)
            .set_dependencies(dependencies))
    }
}
