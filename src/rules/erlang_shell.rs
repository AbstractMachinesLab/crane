use crate::build::{BuildContext, BuildRule, Rule};
use crate::label::Label;
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

    fn run(&mut self, ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        let code_paths: HashSet<PathBuf> = ctx
            .transitive_dependencies(&self.clone().as_rule())
            .iter()
            .flat_map(|dep| dep.outputs(&ctx))
            .flat_map(|artifact| artifact.outputs)
            .map(|path| path.parent().unwrap().to_path_buf())
            .collect();

        ctx.toolchain()
            .erlang()
            .shell(code_paths.into_iter().collect())
    }

    fn build(&mut self, _ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
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
