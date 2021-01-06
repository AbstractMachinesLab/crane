use super::{Action, ComputedTarget, ConfigSpec, Label, RuleConfig, ToolchainManager};
use anyhow::*;
use std::collections::HashMap;
use std::path::PathBuf;

pub type RuleName = String;

/// A Rule defines what actions to take to perform some work.
///
/// Some examples of rules are `ErlangLibrary` or `ElixirTest`.
///
#[derive(Debug, Clone)]
pub struct Rule {
    /// The name of this rule.
    ///
    /// Rule names are unique in a workspace.
    ///
    name: RuleName,

    /// A pretty name to print while running this rule.
    mnemonic: String,

    /// The toolchains this tool depends on.
    toolchains: Vec<Label>,

    /// The rule's configuration map.
    ///
    /// These are the things that a user can pass in when configuring their target,
    /// and it will always support at least `name: String`.
    ///
    cfg: ConfigSpec,

    /// A map of default configuration values.
    defaults: RuleConfig,
}

impl Rule {
    pub fn new(
        name: RuleName,
        mnemonic: String,
        toolchains: Vec<Label>,
        cfg: ConfigSpec,
        defaults: RuleConfig,
    ) -> Rule {
        Rule {
            name,
            mnemonic,
            toolchains,
            cfg,
            defaults,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mnemonic(&self) -> &str {
        &self.mnemonic
    }

    pub fn config(&self) -> &ConfigSpec {
        &self.cfg
    }

    pub fn defaults(&self) -> &RuleConfig {
        &self.defaults
    }
}
