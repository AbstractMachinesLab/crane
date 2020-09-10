use crate::build::{Artifact, BuildContext, BuildRule, Rule};
use crate::label::Label;
use anyhow::{anyhow, Context};
use glob::glob;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::path::PathBuf;
use toml::Value;

#[derive(Debug, Clone)]
pub struct ErlangLibrary {
    name: Label,
    sources: Vec<PathBuf>,
    headers: Vec<PathBuf>,
    dependencies: Vec<Label>,
    outputs: Vec<PathBuf>,
}

impl ErlangLibrary {
    pub fn set_sources(&self, sources: Vec<PathBuf>) -> ErlangLibrary {
        ErlangLibrary {
            sources,
            ..self.clone()
        }
    }

    pub fn set_headers(&self, headers: Vec<PathBuf>) -> ErlangLibrary {
        ErlangLibrary {
            headers,
            ..self.clone()
        }
    }

    pub fn sources(&self) -> Vec<PathBuf> {
        self.sources.clone()
    }

    pub fn headers(&self) -> Vec<PathBuf> {
        self.headers.clone()
    }
}

impl Rule for ErlangLibrary {
    fn as_rule(self) -> BuildRule {
        BuildRule::ErlangLibrary(self)
    }

    fn new(name: Label) -> ErlangLibrary {
        ErlangLibrary {
            name,
            sources: vec![],
            headers: vec![],
            dependencies: vec![],
            outputs: vec![],
        }
    }

    fn set_name(&self, name: Label) -> ErlangLibrary {
        ErlangLibrary {
            name,
            ..self.clone()
        }
    }

    fn set_dependencies(&self, dependencies: Vec<Label>) -> ErlangLibrary {
        ErlangLibrary {
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
        vec![self.headers.clone(), self.sources.clone()]
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
                .chain(self.headers.clone())
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

        let headers: Vec<PathBuf> = self
            .headers
            .iter()
            .cloned()
            .map(|f| ctx.declare_output(f))
            .chain(transitive_headers)
            .collect();

        let files: Vec<PathBuf> = self.headers.to_vec();

        files.iter().cloned().for_each(|f| {
            ctx.declare_output(f);
        });

        ctx.copy(&files).and_then(|_| {
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
                    .map(|f| ctx.declare_output(f.with_extension("beam")))
                    .collect();

                let beam_files: Vec<PathBuf> = beam_files
                    .iter()
                    .chain(transitive_beam_files.iter())
                    .cloned()
                    .collect();

                let dest = beam_files[0].clone();

                ctx.toolchain()
                    .erlang()
                    .compile(&self.sources, &headers, &beam_files, &dest)
            } else {
                Ok(())
            }
        })
    }
}

impl TryFrom<(toml::Value, &PathBuf)> for ErlangLibrary {
    type Error = anyhow::Error;

    fn try_from(input: (toml::Value, &PathBuf)) -> Result<ErlangLibrary, anyhow::Error> {
        let (lib, path) = input;
        let name = lib
            .get("name")
            .context("Rule does not have a valid name")?
            .as_str()
            .context("Names should always be strings")?
            .to_string();
        let name: Label = name.into();
        let name = name.canonicalize(&path);

        let headers = match &lib.get("headers").unwrap_or(&Value::Array(vec![
            Value::String("*.hrl".to_string()),
            Value::String("include/*.hrl".to_string()),
        ])) {
            Value::Array(headers) => headers
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

        let sources = match &lib.get("sources").unwrap_or(&Value::Array(vec![
            Value::String("*.erl".to_string()),
            Value::String("src/*.erl".to_string()),
        ])) {
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

        Ok(ErlangLibrary::new(name)
            .set_sources(sources)
            .set_headers(headers)
            .set_dependencies(dependencies))
    }
}
