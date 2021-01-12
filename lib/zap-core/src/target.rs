use super::{Archive, Label, Rule, RuleConfig};

/// A Target in the Zap dependency graph is a labeled instantiation of a rule plus a configuration
/// object.
///
/// Global targets normally have an Archive as well, such as for Toolchains that need to be
/// downloaded and compiled.
///
#[derive(Debug, Clone)]
pub enum Target {
    /// Local targets are created via build files in a workspace.
    ///
    /// These are not shared with any workspace.
    ///
    Local(LocalTarget),

    /// Global targets are created via toolchains and archives and are built once
    /// and cached across workspaces.
    ///
    Global(GlobalTarget),
}

#[derive(Debug, Clone)]
pub struct LocalTarget {
    /// The name of this local target.
    label: Label,

    /// The dependencies of this target.
    deps: Vec<Label>,

    /// The rule used to build this target.
    rule: Rule,

    /// The target's configuration. To be type-checked against the rule.
    cfg: RuleConfig,
}

#[derive(Debug, Clone)]
pub struct GlobalTarget {
    /// The name of this global target.
    label: Label,

    /// The rule used to build this target.
    rule: Rule,

    /// The target's configuration. To be type-checked against the rule.
    cfg: RuleConfig,

    /// Toolchain targets will have an archive that we need to download
    archive: Archive,
}

impl Target {
    pub fn global(label: Label, rule: &Rule, cfg: RuleConfig, archive: Archive) -> Target {
        Target::Global(GlobalTarget {
            cfg,
            label,
            archive,
            rule: rule.clone(),
        })
    }

    pub fn local(label: Label, rule: &Rule, cfg: RuleConfig) -> Target {
        let mut deps = cfg.get_label_list("deps").unwrap_or_default();
        deps.extend_from_slice(rule.toolchains());

        Target::Local(LocalTarget {
            deps,
            cfg,
            label,
            rule: rule.clone(),
        })
    }

    pub fn is_local(&self) -> bool {
        match self {
            Target::Local(_) => true,
            _ => false,
        }
    }

    pub fn is_global(&self) -> bool {
        match self {
            Target::Global(_) => true,
            _ => false,
        }
    }

    pub fn config(&self) -> &RuleConfig {
        match self {
            Target::Global(t) => &t.cfg,
            Target::Local(t) => &t.cfg,
        }
    }

    pub fn rule(&self) -> &Rule {
        match self {
            Target::Global(t) => &t.rule,
            Target::Local(t) => &t.rule,
        }
    }

    pub fn label(&self) -> &Label {
        match self {
            Target::Global(t) => &t.label,
            Target::Local(t) => &t.label,
        }
    }

    pub fn deps(&self) -> Vec<Label> {
        match self {
            Target::Global(_) => vec![],
            Target::Local(t) => t.deps.clone(),
        }
    }

    pub fn archive(&self) -> Option<&Archive> {
        match self {
            Target::Global(t) => Some(&t.archive),
            Target::Local(_) => None,
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}(name = \"{}\")",
            self.rule().mnemonic(),
            self.label().to_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn uses_rule_mnemonic_to_print_itself() {
        let label = Label::new("test_target");
        let rule = Rule::new(
            "test_rule".to_string(),
            "TestRule".to_string(),
            vec![],
            ConfigSpec::default(),
            RuleConfig::default(),
        );
        let cfg = RuleConfig::default();
        let target = Target::local(label, &rule, cfg);
        assert_eq!("TestRule(name = \":test_target\")", format!("{}", target));
    }

    #[test]
    fn includes_rule_toolchains_in_dependencies() {
        let label = Label::new("test_target");
        let rule = Rule::new(
            "test_rule".to_string(),
            "TestRule".to_string(),
            vec![Label::new("dep")],
            ConfigSpec::default(),
            RuleConfig::default(),
        );
        let cfg = RuleConfig::default();
        let target = Target::local(label, &rule, cfg);
        assert_eq!(1, target.deps().len());
        assert_eq!(Label::new("dep"), target.deps()[0]);
    }
}
