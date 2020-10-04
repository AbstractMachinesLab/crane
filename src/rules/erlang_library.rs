use crate::build::{Artifact, BuildPlan, BuildRule, Rule};
use crate::label::Label;
use crate::toolchains::Toolchains;
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

    fn inputs(&self) -> Vec<PathBuf> {
        vec![self.headers.clone(), self.sources.clone()]
            .iter()
            .flatten()
            .cloned()
            .collect()
    }

    fn outputs(&self) -> Vec<Artifact> {
        vec![Artifact {
            inputs: self.inputs(),
            outputs: self
                .sources
                .iter()
                .map(|file| file.with_extension("beam"))
                .chain(self.headers.clone())
                .collect(),
        }]
    }

    fn build(&mut self, plan: &BuildPlan, toolchains: &Toolchains) -> Result<(), anyhow::Error> {
        // Collect all the transitive dependencies' outputs
        let transitive_deps: Vec<PathBuf> = plan
            .find_nodes(&self.dependencies())
            .iter()
            .flat_map(|node| node.outs())
            .flat_map(|artifact| artifact.outputs.clone())
            .collect();

        // Collect all the headers
        let transitive_headers: HashSet<&PathBuf> = transitive_deps
            .iter()
            .filter(|out| {
                out.extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .map(|str| str.eq_ignore_ascii_case("hrl"))
                    .unwrap_or(false)
            })
            .collect();

        let headers: Vec<PathBuf> = self
            .headers
            .iter()
            .chain(transitive_headers)
            .cloned()
            .collect();

        // Collect all the sources
        let transitive_sources: HashSet<&PathBuf> = transitive_deps
            .iter()
            .filter(|out| {
                out.extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .map(|str| str.eq_ignore_ascii_case("erl"))
                    .unwrap_or(false)
            })
            .collect();

        let sources: Vec<PathBuf> = self
            .sources
            .iter()
            .chain(transitive_sources)
            .cloned()
            .collect();

        // Collect all the bea files
        let transitive_beam_files: HashSet<&PathBuf> = transitive_deps
            .iter()
            .filter(|out| {
                out.extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .map(|str| str.eq_ignore_ascii_case("beam"))
                    .unwrap_or(false)
            })
            .collect();

        let beam_files: Vec<PathBuf> = vec![]
            .iter()
            .chain(transitive_beam_files)
            .cloned()
            .collect();

        toolchains.erlang().compile(&sources, &headers, &beam_files)
    }
}

impl TryFrom<(toml::Value, &PathBuf)> for ErlangLibrary {
    type Error = anyhow::Error;

    fn try_from(input: (toml::Value, &PathBuf)) -> Result<ErlangLibrary, anyhow::Error> {
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

        let sources = match &lib.get("sources").unwrap_or(&Value::Array(vec![])) {
            Value::Array(sources) => sources
                .iter()
                .flat_map(|f| match f {
                    Value::String(name) =>  {
                        if (Label::is_label_like(name)) {
                            let label: Label = x.to_string().into();
                            vec![ label.canonicalize(&path) ]
                        } else {
                        glob(path.join(name).to_str().unwrap())
                        .expect("Could not read glob")
                        .filter_map(Result::ok)
                        .collect(),
                        }
                },
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

        Ok(ErlangLibrary::new(name)
            .set_sources(sources)
            .set_headers(headers)
            .set_dependencies(dependencies))
    }
}
