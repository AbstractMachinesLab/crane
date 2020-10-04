use super::{Artifact, BuildPlan};
use crate::label::Label;
use crate::rules::*;
use crate::toolchains::{ToolchainName, Toolchains};
use anyhow::anyhow;
use std::path::PathBuf;

/// A BuildRule defines what actions to be taken to compute a BuildNode.
///
/// NOTE(@ostera): this could've been a trait but the petgraph library insisted in
/// having Sized implemente for the graph nodes.
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

    pub fn run(&mut self, plan: &BuildPlan, toolchains: &Toolchains) -> Result<(), anyhow::Error> {
        match self {
            BuildRule::ClojureLibrary(lib) => lib.run(plan, toolchains),
            BuildRule::ElixirLibrary(lib) => lib.run(plan, toolchains),
            BuildRule::ErlangLibrary(lib) => lib.run(plan, toolchains),
            BuildRule::ErlangShell(shell) => shell.run(plan, toolchains),
            BuildRule::GleamLibrary(lib) => lib.run(plan, toolchains),
            BuildRule::CaramelLibrary(lib) => lib.run(plan, toolchains),
            BuildRule::Noop => Ok(()),
        }
    }

    pub fn build(
        &mut self,
        plan: &BuildPlan,
        toolchains: &Toolchains,
    ) -> Result<(), anyhow::Error> {
        match self {
            BuildRule::ClojureLibrary(lib) => lib.build(plan, toolchains),
            BuildRule::ElixirLibrary(lib) => lib.build(plan, toolchains),
            BuildRule::ErlangLibrary(lib) => lib.build(plan, toolchains),
            BuildRule::ErlangShell(shell) => shell.build(plan, toolchains),
            BuildRule::GleamLibrary(lib) => lib.build(plan, toolchains),
            BuildRule::CaramelLibrary(lib) => lib.build(plan, toolchains),
            BuildRule::Noop => Ok(()),
        }
    }

    pub fn inputs(&self) -> Vec<PathBuf> {
        match self {
            BuildRule::ClojureLibrary(lib) => lib.inputs(),
            BuildRule::ElixirLibrary(lib) => lib.inputs(),
            BuildRule::ErlangLibrary(lib) => lib.inputs(),
            BuildRule::ErlangShell(shell) => shell.inputs(),
            BuildRule::GleamLibrary(lib) => lib.inputs(),
            BuildRule::CaramelLibrary(lib) => lib.inputs(),
            BuildRule::Noop => vec![],
        }
    }

    pub fn outputs(&self) -> Vec<Artifact> {
        match self {
            BuildRule::ClojureLibrary(lib) => lib.outputs(),
            BuildRule::ElixirLibrary(lib) => lib.outputs(),
            BuildRule::ErlangLibrary(lib) => lib.outputs(),
            BuildRule::ErlangShell(shell) => shell.outputs(),
            BuildRule::GleamLibrary(lib) => lib.outputs(),
            BuildRule::CaramelLibrary(lib) => lib.outputs(),
            BuildRule::Noop => vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum Input {
    SourceFile(PathBuf),
    Computed(Label),
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

    /// The inputs of a build rule are the collection of files used as compilation input.  If a
    /// rule depends on anything else, it should be explicit about it and declare it in this
    /// function.
    ///
    /// Inputs that are not listed here, will simply not be sandboxed, and will not be available
    /// during the build rule execution.
    ///
    /// Absolute path inputs trying to escape the sandbox, will be able to do so in local builds,
    /// but will fail to find these paths on remote builds.
    ///
    /// So to be a good citizen of Crane, keep your inputs declared, and relative to the build rule
    /// location.
    fn inputs(&self) -> Vec<PathBuf> {
        vec![]
    }

    /// The outputs of a build rule are a collecton of Artifact definitions, where an Artifact
    /// establishes the relation between one or more inputs to one ore more sources.
    ///
    /// If a compiler can compile individual inputs separately, but they have been bundled into a
    /// library for convenience, the right thing to do here is to have a vector where each input
    /// has been turned into an artifact, specifying its corresponding compilation output.
    ///
    /// If a compiler can establish a build-time dependency between inputs and will not allow them
    /// to be compiled individually (e.g, it supports circular dependencies between them), then it
    /// is acceptable to return a single Artifact where both inputs are mapped to the corresponding
    /// outputs.
    fn outputs(&self) -> Vec<Input> {
        vec![]
    }

    fn run(&mut self, _plan: &BuildPlan, _toolchain: &Toolchains) -> Result<(), anyhow::Error> {
        Err(anyhow!("This rule does not implement Rule::run/2."))
    }

    fn build(&mut self, _plan: &BuildPlan, _toolchain: &Toolchains) -> Result<(), anyhow::Error> {
        Err(anyhow!("This rule does not implement Rule::build/2."))
    }
}
