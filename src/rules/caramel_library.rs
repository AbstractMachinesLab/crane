use crate::build::{Artifact, BuildContext, BuildRule, Rule};
use crate::label::Label;
use anyhow::{anyhow, Context};
use glob::glob;
use log::debug;
use std::convert::TryFrom;
use std::path::PathBuf;
use toml::Value;

#[derive(Debug, Clone)]
pub struct CaramelLibrary {
    name: Label,
    sources: Vec<PathBuf>,
    dependencies: Vec<Label>,
    outputs: Vec<PathBuf>,
}

impl CaramelLibrary {
    pub fn set_sources(&self, sources: Vec<PathBuf>) -> CaramelLibrary {
        CaramelLibrary {
            sources,
            ..self.clone()
        }
    }

    pub fn sources(&self) -> Vec<PathBuf> {
        self.sources.clone()
    }
}

impl Rule for CaramelLibrary {
    fn as_rule(self) -> BuildRule {
        BuildRule::CaramelLibrary(self)
    }

    fn new(name: Label) -> CaramelLibrary {
        CaramelLibrary {
            name,
            sources: vec![],
            dependencies: vec![],
            outputs: vec![],
        }
    }

    fn set_name(&self, name: Label) -> CaramelLibrary {
        CaramelLibrary {
            name,
            ..self.clone()
        }
    }

    fn set_dependencies(&self, dependencies: Vec<Label>) -> CaramelLibrary {
        CaramelLibrary {
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

    // FIXME(@ostera): map inputs to artifacts instead
    fn outputs(&self, ctx: &BuildContext) -> Vec<Artifact> {
        vec![Artifact {
            inputs: self.inputs(&ctx),
            outputs: self
                .sources
                .iter()
                .flat_map(|file| {
                    vec![
                        file.with_extension("cmi"),
                        file.with_extension("cmo"),
                        file.with_extension("erl"),
                    ]
                })
                .collect(),
        }]
    }

    fn build(&mut self, ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        ctx.toolchain().caramel().compile(&self.inputs(ctx))
    }
}

impl TryFrom<(toml::Value, &PathBuf)> for CaramelLibrary {
    type Error = anyhow::Error;

    fn try_from(input: (toml::Value, &PathBuf)) -> Result<CaramelLibrary, anyhow::Error> {
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

        let sources = match &lib.get("sources").unwrap_or(&Value::Array(vec![
            Value::String("*.ml".to_string()),
            Value::String("*.mli".to_string()),
        ])) {
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

        Ok(CaramelLibrary::new(name)
            .set_sources(sources)
            .set_dependencies(dependencies))
    }
}
