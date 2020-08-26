use crate::build_plan::BuildContext;
use crate::build_rules::library::Library;
use crate::build_rules::shell::Shell;
use crate::build_rules::test::Test;
use crate::model::target::Label;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum BuildRule {
    // Application(Application),
    // Binary(Binary),
    // Config(Config),
    // Project(Project),
    // Release(Release),
    // Template(Template),
    // Toolchain(Toolchain),
    Library(Library),
    Noop,
    Shell(Shell),
    Test(Test),
}

impl Default for BuildRule {
    fn default() -> BuildRule {
        BuildRule::Noop
    }
}

impl BuildRule {
    pub fn run(&self, build_ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        match self {
            BuildRule::Noop => Ok(()),
            BuildRule::Library(library) => library.run(),
            BuildRule::Shell(shell) => shell.run(build_ctx),
            BuildRule::Test(test) => test.run(),
        }
    }

    pub fn build(&self, build_ctx: &mut BuildContext) -> Result<(), anyhow::Error> {
        match self {
            BuildRule::Noop => Ok(()),
            BuildRule::Library(library) => library.build(build_ctx),
            BuildRule::Shell(shell) => shell.build(),
            BuildRule::Test(test) => test.build(),
        }
    }

    pub fn inputs(&self) -> Vec<PathBuf> {
        match self {
            BuildRule::Noop => vec![],
            BuildRule::Library(library) => library.inputs(),
            BuildRule::Shell(shell) => shell.inputs(),
            BuildRule::Test(test) => test.inputs(),
        }
    }

    pub fn outputs(&self, ctx: &BuildContext) -> Vec<PathBuf> {
        match self {
            BuildRule::Noop => vec![],
            BuildRule::Library(library) => library.outputs(ctx),
            BuildRule::Shell(shell) => shell.outputs(ctx),
            BuildRule::Test(test) => test.outputs(),
        }
    }
    pub fn dependencies(&self) -> Vec<Label> {
        match self {
            BuildRule::Noop => vec![],
            BuildRule::Library(library) => library.dependencies(),
            BuildRule::Shell(shell) => shell.dependencies(),
            BuildRule::Test(test) => test.dependencies(),
        }
    }

    pub fn label(&self) -> Label {
        match self {
            BuildRule::Noop => Label::default(),
            BuildRule::Library(library) => library.name(),
            BuildRule::Shell(shell) => shell.name(),
            BuildRule::Test(test) => test.name.clone(),
        }
    }
}
