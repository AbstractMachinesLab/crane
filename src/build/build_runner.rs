use super::{BuildCache, BuildPlan, Sandbox, ValidationStatus};
use crate::toolchains::{ToolchainName, Toolchains};
use crate::workspace::Workspace;
use anyhow::{anyhow, Context};
use log::debug;
use petgraph::visit::Topo;

/// The BuildRunner is in charge of actually executing a BuildPlan in the
/// context of a Workspace, using a given Toolchain, and a given BuildCache.
///
/// This struct essentially has the core logic that defines how the system
/// builds your projects.
///
/// It can:
///
/// 1. Ready all the relevant toolchains for this particular BuildPlan
/// //2. Iterate over the BuildPlan, executing runnable rules
/// 3. Iterate over the BuildPlan, executing buildable rules in a Sandbox,
///    and updatting the Cache accordingly
///
#[derive(Debug, Clone)]
pub struct BuildRunner {
    /// The workspace in which the build runner will execute.
    workspace: Workspace,

    /// The build plan to execute.
    build_plan: BuildPlan,

    /// The build cache to save build results to.
    build_cache: BuildCache,

    /// The toolchain to execute build rules with.
    toolchains: Toolchains,
}

impl BuildRunner {
    pub fn new(workspace: Workspace, build_plan: BuildPlan) -> BuildRunner {
        BuildRunner {
            build_plan,
            build_cache: BuildCache::new(workspace.clone()),
            toolchains: workspace.clone().toolchains(),
            workspace,
        }
    }

    pub fn run(&mut self) -> Result<u32, anyhow::Error> {
        let mut targets = 0;

        let mut bg = self.build_plan.build_graph.clone();
        bg.reverse();
        let mut walker = Topo::new(&bg);
        while let Some(node) = walker.next(&bg) {
            let node = &mut bg[node];
            let name = node.rule().name();
            debug!("About to run {:?}...", name.to_string());
            node.rule().run(&self.build_plan, &self.toolchains)?;
            targets += 1;
        }

        Ok(targets)
    }

    pub fn build(&mut self) -> Result<u32, anyhow::Error> {
        let mut targets = 0;

        self.build_plan.build_graph.reverse();
        let mut walker = Topo::new(&self.build_plan.build_graph);

        while let Some(idx) = walker.next(&self.build_plan.build_graph) {
            let node = &self.build_plan.build_graph[idx];
            // NOTE: unfortunately passing in a reference to the build_plan into
            // the update_hash function means the mutable borrow that defines the
            // node itself needs to be broken.
            let deps = self.build_plan.find_nodes(&node.deps());

            let node = &mut self.build_plan.build_graph[idx];
            node.update_hash(deps);

            let node = &self.build_plan.build_graph[idx];

            let name = node.name();
            debug!("About to build {:?}...", name.to_string());
            if self.build_cache.is_cached(&node)? {
                debug!("Skipping {}. Nothing to do.", name.to_string());
            } else {
                let mut sandbox = Sandbox::for_node(self.workspace.clone(), node.clone());
                match sandbox.run(&self.build_plan, &self.build_cache, &self.toolchains)? {
                    ValidationStatus::Valid => {
                        let node = &mut self.build_plan.build_graph[idx];
                        *node = sandbox.node().clone();
                        self.build_cache.save(&sandbox)?;
                        targets += 1;
                        Ok(())
                    },
                    ValidationStatus::NoOutputs => {
                        Err(anyhow!("Node {} had no outputs...was the sandbox cleaned while the build was running?", &name.to_string()))
                    },
                    ValidationStatus::Pending => {
                        Err(anyhow!("Node {} is somehow still pending...", &name.to_string()))
                    },
                    ValidationStatus::Invalid { expected_but_missing, unexpected_but_present, } => {
                        Err(anyhow!("Node {} expected the following but missing outputs: {:?}\n\ninstead it found the following unexpected outputs: {:?}", &name.to_string(), expected_but_missing, unexpected_but_present))
                    },
                }?
            }
        }

        Ok(targets)
    }

    pub fn ready_toolchains(&mut self) -> Result<(), anyhow::Error> {
        let home = home::home_dir().context("Could not get your home directory, is HOME set?")?;
        let dotcrane = home.join(".crane");
        let toolchains_dir = dotcrane.join("toolchains");
        std::fs::create_dir_all(&toolchains_dir).context(format!(
            "Failed to create toolchains folder at {:?}",
            &toolchains_dir
        ))?;

        self.toolchains = self.toolchains.clone().set_root(toolchains_dir);

        // NOTE(@ostera): this will not be the primary toolchain if Lumen support
        // is added!
        &self.toolchains.ready_toolchain(ToolchainName::Erlang)?;

        let ts: Vec<ToolchainName> = self
            .build_plan
            .toolchains_in_use()
            .iter()
            .filter(|n| ToolchainName::Erlang.ne(n))
            .cloned()
            .collect();
        for t in ts {
            self.toolchains.ready_toolchain(t)?
        }

        Ok(())
    }
}
