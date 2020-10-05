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
pub struct LumenBinary {
    name: Label,
    sources: Vec<PathBuf>,
    headers: Vec<PathBuf>,
    dependencies: Vec<Label>,
    outputs: Vec<PathBuf>,
}

impl LumenBinary {
    pub fn set_sources(&self, sources: Vec<PathBuf>) -> LumenBinary {
        LumenBinary {
            sources,
            ..self.clone()
        }
    }

    pub fn set_headers(&self, headers: Vec<PathBuf>) -> LumenBinary {
        LumenBinary {
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

impl Rule for LumenBinary {
    fn as_rule(self) -> BuildRule {
        BuildRule::LumenBinary(self)
    }

    fn new(name: Label) -> LumenBinary {
        LumenBinary {
            name,
            sources: vec![],
            headers: vec![],
            dependencies: vec![],
            outputs: vec![],
        }
    }

    fn set_name(&self, name: Label) -> LumenBinary {
        LumenBinary {
            name,
            ..self.clone()
        }
    }

    fn set_dependencies(&self, dependencies: Vec<Label>) -> LumenBinary {
        LumenBinary {
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
        vec![self.headers.clone(), self.sources.clone()]
            .iter()
            .flatten()
            .cloned()
            .collect()
    }

    fn set_inputs(self, sources: Vec<PathBuf>) -> LumenBinary {
        LumenBinary { sources, ..self }
    }

    fn outputs(&self, deps: &[Artifact]) -> Vec<Artifact> {
        let extra_sources: Vec<PathBuf> = deps
            .iter()
            .flat_map(|a| a.outputs.clone())
            .filter(|path| {
                path.extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .map(|str| str.eq_ignore_ascii_case("erl"))
                    .unwrap_or(false)
            })
            .collect();

        vec![Artifact {
            inputs: self.inputs(&deps),
            outputs: self
                .sources
                .iter()
                .cloned()
                .chain(extra_sources)
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

        toolchains.lumen().compile(&sources)
    }
}

impl TryFrom<(toml::Value, &PathBuf)> for LumenBinary {
    type Error = anyhow::Error;

    fn try_from(input: (toml::Value, &PathBuf)) -> Result<LumenBinary, anyhow::Error> {
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

        let sources = match &lib
            .get("srcs")
            .unwrap_or(&Value::Array(vec![Value::String("*.erl".to_string())]))
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

        Ok(LumenBinary::new(name)
            .set_sources(sources)
            .set_headers(headers)
            .set_dependencies(dependencies))
    }
}
