use anyhow::*;
use std::path::PathBuf;
use structopt::StructOpt;
use zap_core::*;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "rule",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "managing rules"
)]
pub enum RulesGoal {
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
    pub async fn run(self, config: ZapConfig) -> Result<(), anyhow::Error> {
        let mut zap = ZapWorker::new(config)?;
        zap.load(&PathBuf::from(&".")).await?;
        zap.build_dep_graph()?;

        match self {
            RulesGoal::List => self.list_rules(&mut zap),
            RulesGoal::DumpActions { ref target } => self.dump_actions(&target, &mut zap),
            RulesGoal::DumpOutputs { ref target } => self.dump_outputs(&target, &mut zap),
        }
    }

    fn list_rules(&self, zap: &mut ZapWorker) -> Result<(), anyhow::Error> {
        let rule_manager = zap.rule_manager();
        println!("Loaded Rules: ");
        let mut rules = rule_manager.read().unwrap().rules();
        rules.sort_by_key(|r| r.name().to_string());
        for rule in rules {
            println!("* {} @ {}", rule.mnemonic(), rule.name().to_string());
        }
        Ok(())
    }

    fn dump_outputs(&self, target: &str, zap: &mut ZapWorker) -> Result<(), anyhow::Error> {
        let label: Label = target.into();
        let dep_graph = &mut zap.dep_graph.scoped(&label)?.seal(
            &zap.action_map,
            &zap.output_map,
            &mut zap.bs_ctx,
        )?;

        for computed_target in dep_graph.targets() {
            for output in computed_target.outs() {
                println!("{}", output.to_str().unwrap());
            }
        }

        Ok(())
    }

    fn dump_actions(&self, target: &str, zap: &mut ZapWorker) -> Result<(), anyhow::Error> {
        let label: Label = target.into();
        let dep_graph = &mut zap.dep_graph.scoped(&label)?.seal(
            &zap.action_map,
            &zap.output_map,
            &mut zap.bs_ctx,
        )?;

        for computed_target in dep_graph.targets() {
            for action in computed_target.actions() {
                println!("{:?}", action);
            }
        }

        Ok(())
    }
}
