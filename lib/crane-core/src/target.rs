use super::{Artifact, DepGraph, Label, Rule};
use std::path::PathBuf;

/// A target in the Crane build plan.
///
pub trait Target {
    fn name(&self) -> &Label;

    fn rule(&self) -> &dyn Rule;

    fn deps(&self) -> &[Label];

    fn set_deps(&mut self, deps: &[Label]);

    /// With the support of a dependency graph, retrieve the list of transitive
    /// dependencies in build order.
    ///
    fn transitive_deps<'a>(&self, plan: &'a DepGraph) -> Vec<&'a Box<dyn Target>> {
        plan.find_nodes(&self.deps())
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
    ///
    fn srcs(&self, _deps: &[Artifact]) -> &[PathBuf];

    fn set_srcs(&mut self, sources: &[PathBuf]);

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
    ///
    fn outputs(&self, _deps: &[Artifact]) -> &[Artifact];
}

impl Default for Box<dyn Target> {
    fn default() -> Self {
        todo!()
    }
}

impl std::fmt::Debug for Box<dyn Target> {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        todo!()
    }
}
