use super::build_artifact::Artifact;
use super::build_context::BuildContext;
use crate::label::Label;
use crate::rules::*;
use crate::toolchains::ToolchainName;
use anyhow::anyhow;
use std::path::PathBuf;

// TODO(@ostera): this could've been a trait but the petgraph library insisted in
// having Sized implemente for the graph nodes.
#[derive(Debug, Clone)]
pub enum BuildRule {
    Noop,
    ErlangLibrary(ErlangLibrary),
    ErlangShell(ErlangShell),
    ClojureLibrary(ClojureLibrary),
    ElixirLibrary(ElixirLibrary),
    GleamLibrary(GleamLibrary),
    CaramelLibrary(CaramelLibrary),
}

impl Default for BuildRule {
    fn default() -> BuildRule {
        BuildRule::Noop
    }
}

impl BuildRule {
    pub fn name(&self) -> Label {
        match self {
            BuildRule::ClojureLibrary(lib) => lib.name(),
            BuildRule::ElixirLibrary(lib) => lib.name(),
            BuildRule::ErlangLibrary(lib) => lib.name(),
            BuildRule::ErlangShell(shell) => shell.name(),
            BuildRule::GleamLibrary(lib) => lib.name(),
            BuildRule::CaramelLibrary(lib) => lib.name(),
            BuildRule::Noop => Label::default(),
        }
    }

    pub fn toolchain(&self) -> Option<ToolchainName> {
        match self {
            BuildRule::ClojureLibrary(_) => Some(ToolchainName::Clojure),
            BuildRule::ElixirLibrary(_) => Some(ToolchainName::Elixir),
            BuildRule::ErlangLibrary(_) => Some(ToolchainName::Erlang),
            BuildRule::ErlangShell(_) => Some(ToolchainName::Erlang),
            BuildRule::GleamLibrary(_) => Some(ToolchainName::Gleam),
            BuildRule::CaramelLibrary(_) => Some(ToolchainName::Caramel),
            BuildRule::Noop => None,
        }
    }

    pub fn dependencies(&self) -> Vec<Label> {
        match self {
            BuildRule::ClojureLibrary(lib) => lib.dependencies(),
            BuildRule::ElixirLibrary(lib) => lib.dependencies(),
            BuildRule::ErlangLibrary(lib) => lib.dependencies(),
            BuildRule::ErlangShell(shell) => shell.dependencies(),
            BuildRule::GleamLibrary(lib) => lib.dependencies(),
            BuildRule::CaramelLibrary(lib) => lib.dependencies(),
            BuildRule::Noop => vec![],
        }
    }

    pub fn run(&mut self, ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        match self {
            BuildRule::ClojureLibrary(lib) => lib.run(ctx),
            BuildRule::ElixirLibrary(lib) => lib.run(ctx),
            BuildRule::ErlangLibrary(lib) => lib.run(ctx),
            BuildRule::ErlangShell(shell) => shell.run(ctx),
            BuildRule::GleamLibrary(lib) => lib.run(ctx),
            BuildRule::CaramelLibrary(lib) => lib.run(ctx),
            BuildRule::Noop => Ok(()),
        }
    }

    pub fn build(&mut self, ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        match self {
            BuildRule::ClojureLibrary(lib) => lib.build(ctx),
            BuildRule::ElixirLibrary(lib) => lib.build(ctx),
            BuildRule::ErlangLibrary(lib) => lib.build(ctx),
            BuildRule::ErlangShell(shell) => shell.build(ctx),
            BuildRule::GleamLibrary(lib) => lib.build(ctx),
            BuildRule::CaramelLibrary(lib) => lib.build(ctx),
            BuildRule::Noop => Ok(()),
        }
    }

    pub fn inputs(&self, ctx: &BuildContext) -> Vec<PathBuf> {
        match self {
            BuildRule::ClojureLibrary(lib) => lib.inputs(ctx),
            BuildRule::ElixirLibrary(lib) => lib.inputs(ctx),
            BuildRule::ErlangLibrary(lib) => lib.inputs(ctx),
            BuildRule::ErlangShell(shell) => shell.inputs(ctx),
            BuildRule::GleamLibrary(lib) => lib.inputs(ctx),
            BuildRule::CaramelLibrary(lib) => lib.inputs(ctx),
            BuildRule::Noop => vec![],
        }
    }

    pub fn outputs(&self, ctx: &BuildContext) -> Vec<Artifact> {
        match self {
            BuildRule::ClojureLibrary(lib) => lib.outputs(ctx),
            BuildRule::ElixirLibrary(lib) => lib.outputs(ctx),
            BuildRule::ErlangLibrary(lib) => lib.outputs(ctx),
            BuildRule::ErlangShell(shell) => shell.outputs(ctx),
            BuildRule::GleamLibrary(lib) => lib.outputs(ctx),
            BuildRule::CaramelLibrary(lib) => lib.outputs(ctx),
            BuildRule::Noop => vec![],
        }
    }
}

pub trait Rule {
    fn as_rule(self) -> BuildRule;

    fn new(name: Label) -> Self;

    fn set_name(&self, name: Label) -> Self;

    fn set_dependencies(&self, dependencies: Vec<Label>) -> Self;

    fn name(&self) -> Label;

    fn dependencies(&self) -> Vec<Label> {
        vec![]
    }

    fn run(&mut self, _ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        Err(anyhow!("This rule does not implement Rule::run/2."))
    }

    fn build(&mut self, _ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        Err(anyhow!("This rule does not implement Rule::build/2."))
    }

    fn inputs(&self, _ctx: &BuildContext) -> Vec<PathBuf> {
        vec![]
    }

    fn outputs(&self, _ctx: &BuildContext) -> Vec<Artifact> {
        vec![]
    }
}
