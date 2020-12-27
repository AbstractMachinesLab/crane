use super::{BuildCache, BuildGraph, Sandbox, ValidationStatus};
use anyhow::anyhow;
use crane_core::Workspace;
use log::debug;
use petgraph::visit::Topo;

/// The BuildRunner is in charge of actually executing a BuildGraph in the
/// context of a Workspace, using a given Toolchain, and a given BuildCache.
///
/// This struct essentially has the core logic that defines how the system
/// builds your projects.
///
/// It can:
///
/// 1. Ready all the relevant toolchains for this particular BuildGraph
/// //2. Iterate over the BuildGraph, executing runnable rules
/// 3. Iterate over the BuildGraph, executing buildable rules in a Sandbox,
///    and updatting the Cache accordingly
///
pub struct BuildRunner {
    /// The workspace in which the build runner will execute.
    workspace: Workspace,

    /// The build plan to execute.
    build_graph: BuildGraph,

    /// The build cache to save build results to.
    build_cache: BuildCache,
}

impl BuildRunner {
    pub fn new(workspace: Workspace, build_graph: BuildGraph) -> BuildRunner {
        BuildRunner {
            build_graph,
            build_cache: BuildCache::new(workspace.clone()),
            workspace,
        }
    }

    pub fn execute(&mut self) -> Result<u32, anyhow::Error> {
        let mut targets = 0;

        self.build_graph.dep_graph.reverse();
        let mut walker = Topo::new(&self.build_graph.dep_graph);

        while let Some(idx) = walker.next(&self.build_graph.dep_graph) {
            // NOTE: unfortunately passing in a reference to the build_graph into
            // the update_hash function means the mutable borrow that defines the
            // node itself needs to be broken.
            let node = &self.build_graph.dep_graph[idx];
            let deps = self.build_graph.find_dependency_hashes_and_outputs(node);
            let node = &mut self.build_graph.dep_graph[idx];
            node.update_hash(deps);

            // Reborrows
            let node = &self.build_graph.dep_graph[idx];

            let name = node.name();
            debug!("About to build {:?}...", name.to_string());
            debug!("with sources {:?}...", &node.srcs());
            debug!("with dependencies {:?}...", &node.deps());

            let result = if self.build_cache.is_cached(&node)? {
                debug!("Skipping {}. Nothing to do.", name.to_string());
                Ok(())
            } else {
                let deps = &self.build_graph.find_nodes(&node.deps());
                let mut sandbox = Sandbox::for_node(self.workspace.clone(), &node, deps.to_vec());
                match sandbox.run(&self.build_cache)? {
                    ValidationStatus::Valid => {
                        self.build_cache.save(&sandbox)?;
                        sandbox.clear_sandbox()?;
                        targets += 1;
                        Ok(())
                    }
                    ValidationStatus::NoOutputs => {
                        sandbox.clear_sandbox()?;
                        targets += 1;
                        Ok(())
                    }
                    ValidationStatus::Pending => Err(anyhow!(
                        "Node {} is somehow still pending...",
                        &name.to_string()
                    )),
                    ValidationStatus::Invalid {
                        expected_but_missing,
                        unexpected_but_present,
                        ..
                    } => {
                        Err(anyhow!("Node {} expected the following but missing outputs: {:?}\n\ninstead it found the following unexpected outputs: {:?}", &name.to_string(), expected_but_missing, unexpected_but_present))
                    }
                }
            };
            /*

            let node = &mut self.build_graph.dep_graph[idx];
            if result.is_ok() {
                node.mark_succeeded();
            } else {
                node.mark_failed();
            }
            */

            result?
        }

        Ok(targets)
    }

    /*
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
            .build_graph
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
    */
}
