use super::{Label, Rule, RuleConfig};

/// A Target in the Zap dependency graph is a labeled instantiation of a rule plus a configuration
/// object.
///
#[derive(Debug, Clone)]
pub struct Target {
    /// The name of this rule instance.
    label: Label,

    /// The dependencies of this target.
    deps: Vec<Label>,

    /// The rule used to build this target.
    rule: Rule,

    /// The target's configuration. To be type-checked against the rule.
    cfg: RuleConfig,
}

impl Target {
    pub fn new(label: Label, rule: &Rule, cfg: RuleConfig) -> Target {
        let mut deps = cfg.get_label_list("deps").unwrap_or(vec![]);
        deps.extend_from_slice(rule.toolchains());

        Target {
            deps,
            cfg,
            label,
            rule: rule.clone(),
        }
    }

    pub fn config(&self) -> &RuleConfig {
        &self.cfg
    }

    pub fn rule(&self) -> &Rule {
        &self.rule
    }

    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn deps(&self) -> &[Label] {
        &self.deps
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}(name = \"{}\")",
            self.rule.mnemonic(),
            self.label.to_string()
        )
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, ComputedTarget};
    use std::path::PathBuf;

    #[test]
    fn can_debug_print() {
        let target: Box<dyn Target> = Box::new(TestTarget::new());
        assert_eq!(
            "test_rule(name = \":test_target\")",
            format!("{:?}", target)
        );
    }
}
*/
