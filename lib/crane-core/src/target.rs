use super::{Artifact, DepGraph, Label, Rule};
use std::path::PathBuf;

/// A target in the Crane build plan.
///
pub trait Target: CloneUnsized {
    fn name(&self) -> &Label;

    fn rule(&self) -> &dyn Rule;

    fn deps(&self) -> &[Label];

    fn set_deps(&mut self, deps: &[Label]);

    /// With the support of a dependency graph, retrieve the list of transitive
    /// dependencies in build order.
    ///
    fn transitive_deps<'a>(&self, plan: &'a DepGraph) -> Vec<&'a dyn Target> {
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

pub trait CloneUnsized {
    fn clone_boxed(&self) -> Box<dyn Target>;
}

impl<T: Clone + Target + 'static> CloneUnsized for T {
    fn clone_boxed(&self) -> Box<dyn Target> {
        Box::new(self.clone())
    }
}

impl Default for Box<dyn Target> {
    fn default() -> Self {
        panic!("Oops! We have attempted to create a default target. This means the DepGraph was somehow incomplete. This is a bug!");
    }
}

impl std::fmt::Debug for dyn Target {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}(name = \"{}\")",
            self.rule().name(),
            self.name().to_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestRule {}
    impl Rule for TestRule {
        fn name(&self) -> &str {
            "test_rule"
        }
        fn toolchains(&self) -> Vec<Label> {
            vec![]
        }
        fn execute(&self) -> Result<(), anyhow::Error> {
            Ok(())
        }
    }

    #[derive(Debug, Clone)]
    struct TestTarget {
        label: Label,
        deps: Vec<Label>,
        srcs: Vec<PathBuf>,
        outs: Vec<Artifact>,
        rule: TestRule,
    }
    impl TestTarget {
        fn new() -> TestTarget {
            TestTarget {
                label: "test_target".into(),
                deps: vec![],
                srcs: vec![],
                outs: vec![],
                rule: TestRule {},
            }
        }
    }
    impl Target for TestTarget {
        fn name(&self) -> &Label {
            &self.label
        }
        fn rule(&self) -> &dyn Rule {
            &self.rule
        }
        fn deps(&self) -> &[Label] {
            &self.deps
        }
        fn set_deps(&mut self, _deps: &[Label]) {}
        fn srcs(&self, _deps: &[Artifact]) -> &[PathBuf] {
            &self.srcs
        }
        fn set_srcs(&mut self, _srcs: &[PathBuf]) {}
        fn outputs(&self, _deps: &[Artifact]) -> &[Artifact] {
            &self.outs
        }
    }

    #[test]
    fn can_debug_print() {
        let target: Box<dyn Target> = Box::new(TestTarget::new());
        assert_eq!(
            "test_rule(name = \":test_target\")",
            format!("{:?}", target)
        );
    }
}
