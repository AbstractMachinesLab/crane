use crate::build::{Artifact, BuildPlan, BuildRule, Rule};
use crate::label::Label;
use crate::toolchains::Toolchains;
use anyhow::{anyhow, Context};
use std::collections::HashSet;
use std::convert::TryFrom;
use std::path::PathBuf;
use toml::Value;

#[derive(Debug, Clone)]
pub struct ErlangShell {
    name: Label,
    dependencies: Vec<Label>,
}

impl Rule for ErlangShell {
    fn as_rule(self) -> BuildRule {
        BuildRule::ErlangShell(self)
    }

    fn new(name: Label) -> ErlangShell {
        ErlangShell {
            name,
            dependencies: vec![],
        }
    }

    fn set_name(&self, name: Label) -> ErlangShell {
        ErlangShell {
            name,
            ..self.clone()
        }
    }

    fn set_dependencies(&self, dependencies: Vec<Label>) -> ErlangShell {
        ErlangShell {
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
        vec![]
    }

    fn set_inputs(self, _inputs: Vec<PathBuf>) -> ErlangShell {
        self
    }

    fn run(&mut self, plan: &BuildPlan, toolchains: &Toolchains) -> Result<(), anyhow::Error> {
        let code_paths: HashSet<PathBuf> = plan
            .find_nodes(&self.dependencies())
            .iter()
            .flat_map(|node| node.outs())
            .flat_map(|artifact| artifact.outputs.clone())
            .map(|path| path.parent().unwrap().to_path_buf())
            .collect();

        toolchains.erlang().shell(code_paths.into_iter().collect())
    }

    fn build(&mut self, _plan: &BuildPlan, _toolchains: &Toolchains) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

impl TryFrom<(toml::Value, &PathBuf)> for ErlangShell {
    type Error = anyhow::Error;

    fn try_from(input: (toml::Value, &PathBuf)) -> Result<ErlangShell, anyhow::Error> {
        let (lib, path) = input;
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

        Ok(ErlangShell::new(name).set_dependencies(dependencies))
    }
}
