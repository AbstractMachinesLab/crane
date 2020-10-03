use super::build_artifact::Artifact;
use super::build_context::BuildContext;
use crate::label::Label;
use crate::rules::*;
use crate::toolchains::ToolchainName;
use anyhow::anyhow;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::path::PathBuf;

// NOTE(@ostera): this could've been a trait but the petgraph library insisted in
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

    pub fn hash(&self, ctx: &mut BuildContext) -> String {
        match self {
            BuildRule::CaramelLibrary(lib) => lib.hash(ctx),
            BuildRule::ClojureLibrary(lib) => lib.hash(ctx),
            BuildRule::ElixirLibrary(lib) => lib.hash(ctx),
            BuildRule::ErlangLibrary(lib) => lib.hash(ctx),
            BuildRule::ErlangShell(shell) => shell.hash(ctx),
            BuildRule::GleamLibrary(lib) => lib.hash(ctx),
            BuildRule::Noop => "no-hash".to_string(),
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

    /// The hash of a rule identifies a rule's entire configuration:
    /// * listed inputs, and their contents
    /// * listed outputs
    /// * rule name
    ///
    /// FIXME: add a pluggable way to add relevant information to this hash to
    /// use build options in it
    ///
    /// TODO: remove dependency on BuildContext, and force all rules to hash
    /// themselves based on their label paths instead
    fn hash(&self, ctx: &BuildContext) -> String {
        let name = self.name();
        let mut hasher = Sha1::new();
        hasher.input_str(&name.to_string());
        for o in self.outputs(ctx) {
            hasher.input_str(&o.compute_hash(name.path()));
        }
        hasher.result_str()
    }

    fn dependencies(&self) -> Vec<Label> {
        vec![]
    }

    fn run(&mut self, _ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        Err(anyhow!("This rule does not implement Rule::run/2."))
    }

    fn build(&mut self, _ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        Err(anyhow!("This rule does not implement Rule::build/2."))
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
    fn inputs(&self, _ctx: &BuildContext) -> Vec<PathBuf> {
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
    fn outputs(&self, _ctx: &BuildContext) -> Vec<Artifact> {
        vec![]
    }
}
