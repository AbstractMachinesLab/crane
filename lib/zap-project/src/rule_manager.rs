use super::*;
use dashmap::DashMap;
use log::*;
use std::path::PathBuf;
use zap_buildscript::*;
use zap_core::*;

#[derive(Debug, Clone, Default)]
pub struct RuleManager {
    rules: DashMap<RuleName, Rule>,
}

impl RuleManager {
    pub fn new() -> RuleManager {
        RuleManager::default()
    }

    pub async fn load_from_workspace(
        &self,
        workspace: &Workspace,
        mut bs_ctx: &mut BuildScript,
    ) -> Result<(), anyhow::Error> {
        self.load(workspace.root(), &mut bs_ctx).await
    }

    pub async fn load(
        &self,
        root: &PathBuf,
        bs_ctx: &mut BuildScript,
    ) -> Result<(), anyhow::Error> {
        let rules = RuleScanner::scan(&root)?;

        for rulefile in rules {
            trace!("Loading rule: {:?}", rulefile);
            bs_ctx.load(rulefile).await?;
        }
        Ok(())
    }

    pub fn register(&self, rule: Rule) {
        self.rules.insert(rule.name().to_string(), rule);
    }

    pub fn get(&self, name: &RuleName) -> Option<Rule> {
        self.rules.get(name).map(|r| r.value().clone())
    }

    pub fn rules(&self) -> Vec<Rule> {
        self.rules
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}
