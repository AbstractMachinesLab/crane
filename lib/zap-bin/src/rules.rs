use anyhow::*;
use log::*;
use std::path::PathBuf;
use structopt::StructOpt;
use zap_core::*;
use zap_project::*;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "rules",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "managing rules"
)]
pub struct RulesGoal {
    #[structopt(subcommand, help = "the command to run")]
    cmd: Action,
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    setting = structopt::clap::AppSettings::ColoredHelp,
)]
enum Action {
    #[structopt(help = r"List all the local and global Zap rules.")]
    List,
    DumpActions {
        #[structopt(help = r"The target to dump actions for.

    This command is primarily useful to debug what actions will be taken to build
    a particular target.

    It will only print this targret's actions, and not any dependencies.
    ")]
        target: String,
    },
    DumpOutputs {
        #[structopt(help = r"The target to dump outputs for.

    This command is primarily useful to debug what outputs will be created when
    a particular target is built.

    It will only print this target's outputs.
    ")]
        target: String,
    },
}

impl RulesGoal {
    pub async fn run(self) -> Result<(), anyhow::Error> {
        let config = ZapConfig::new()?;
        let mut zap = ZapWorker::new(config)?;
        zap.load(&PathBuf::from(&".")).await?;

        match self.cmd {
            Action::List => self.list_rules(&mut zap),
            Action::DumpActions { ref target } => self.dump_actions(&target, &mut zap),
            Action::DumpOutputs { ref target } => self.dump_outputs(&target, &mut zap),
        }
    }

    fn list_rules(&self, zap: &mut ZapWorker) -> Result<(), anyhow::Error> {
        let rule_manager = zap.rule_manager();
        info!("Loaded Rules: ");
        for rule in rule_manager.read().unwrap().rules() {
            info!("* {} @ {}", rule.mnemonic(), rule.name().to_string());
        }
        Ok(())
    }

    fn dump_outputs(&self, target: &str, zap: &mut ZapWorker) -> Result<(), anyhow::Error> {
        let label: Label = target.into();
        let dep_graph = &mut zap.dep_graph;
        dep_graph.seal_target_by_label(
            &label,
            &zap.action_map,
            &zap.output_map,
            &mut zap.bs_ctx,
        )?;
        let computed_target = dep_graph.find_node(&label).context(format!(
            "Could not find target with label {}",
            label.to_string()
        ))?;

        for output in computed_target.outs() {
            info!("{:?}", output);
        }

        Ok(())
    }

    fn dump_actions(&self, target: &str, zap: &mut ZapWorker) -> Result<(), anyhow::Error> {
        let label: Label = target.into();
        let dep_graph = &mut zap.dep_graph;
        dep_graph.seal_target_by_label(
            &label,
            &zap.action_map,
            &zap.output_map,
            &mut zap.bs_ctx,
        )?;
        let computed_target = dep_graph.find_node(&label).context(format!(
            "Could not find target with label {}",
            label.to_string()
        ))?;

        for action in computed_target.actions() {
            info!("{:?}", action);
        }

        Ok(())
    }
}
