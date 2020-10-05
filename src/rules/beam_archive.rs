use crate::build::{Artifact, BuildPlan, BuildRule, Rule};
use crate::label::Label;
use crate::toolchains::Toolchains;
use anyhow::{anyhow, Context};
use std::convert::TryFrom;
use std::path::PathBuf;
use toml::Value;

#[derive(Debug, Clone)]
pub struct BeamArchive {
    name: Label,
    archive_name: PathBuf,
    dependencies: Vec<Label>,
    outputs: Vec<PathBuf>,
}

impl BeamArchive {
    pub fn set_archive_name(self, archive_name: PathBuf) -> BeamArchive {
        BeamArchive {
            archive_name,
            ..self
        }
    }
}

impl Rule for BeamArchive {
    fn as_rule(self) -> BuildRule {
        BuildRule::BeamArchive(self)
    }

    fn new(name: Label) -> BeamArchive {
        BeamArchive {
            archive_name: PathBuf::from(name.name()).with_extension("tar.gz"),
            name,
            dependencies: vec![],
            outputs: vec![],
        }
    }

    fn set_name(&self, name: Label) -> BeamArchive {
        BeamArchive {
            name,
            ..self.clone()
        }
    }

    fn set_dependencies(&self, dependencies: Vec<Label>) -> BeamArchive {
        BeamArchive {
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
        vec![]
    }

    fn outputs(&self, _deps: &[Artifact]) -> Vec<Artifact> {
        vec![Artifact {
            inputs: vec![],
            outputs: vec![self.archive_name.clone()],
        }]
    }

    fn set_inputs(self, _sources: Vec<PathBuf>) -> BeamArchive {
        self
    }

    fn build(&mut self, plan: &BuildPlan, toolchains: &Toolchains) -> Result<(), anyhow::Error> {
        let transitive_deps: Vec<PathBuf> = plan
            .find_nodes(&self.dependencies())
            .iter()
            .flat_map(|node| node.outs())
            .flat_map(|artifact| artifact.outputs.clone())
            .collect();
        toolchains
            .standard()
            .tar(&self.archive_name, &transitive_deps)
    }
}

impl TryFrom<(toml::Value, &PathBuf)> for BeamArchive {
    type Error = anyhow::Error;

    fn try_from(input: (toml::Value, &PathBuf)) -> Result<BeamArchive, anyhow::Error> {
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

        Ok(BeamArchive::new(name).set_dependencies(dependencies))
    }
}
