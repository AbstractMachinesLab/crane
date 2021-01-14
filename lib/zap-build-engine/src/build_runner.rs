use super::{BuildCache, CacheHitType, Sandbox, ValidationStatus};
use anyhow::anyhow;
use dashmap::DashMap;
use log::debug;
use petgraph::visit::Topo;
use std::path::PathBuf;
use std::sync::Arc;
use zap_buildscript::*;
use zap_core::{Action, DepGraph, Label, Workspace, ZapConfig, ZapWorker};

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

    /// The dependency graph
    dep_graph: DepGraph,

    /// The build cache to save build results to.
    build_cache: BuildCache,

    action_map: Arc<DashMap<Label, Vec<Action>>>,

    output_map: Arc<DashMap<Label, Vec<PathBuf>>>,

    bs_ctx: BuildScript,

    config: ZapConfig,
}

impl BuildRunner {
    pub fn new(zap: ZapWorker) -> BuildRunner {
        BuildRunner {
            action_map: zap.action_map,
            output_map: zap.output_map,
            bs_ctx: zap.bs_ctx,
            dep_graph: zap.dep_graph,
            build_cache: BuildCache::new(&zap.config),
            workspace: zap.workspace,
            config: zap.config,
        }
    }

    pub fn execute(&mut self, target: &Label) -> Result<u32, anyhow::Error> {
        self.dep_graph.scoped(&target)?;

        let mut targets = 0;

        let mut walker = Topo::new(&self.dep_graph._inner_graph);

        while let Some(idx) = walker.next(&self.dep_graph._inner_graph) {
            let node = &self.dep_graph.seal_target(
                idx,
                &self.action_map,
                &self.output_map,
                &mut self.bs_ctx,
            )?;

            let name = node.label().clone();
            debug!("About to build {:?}...", name.to_string());
            debug!("with sources {:?}...", &node.srcs());
            debug!("with dependencies {:?}...", &node.deps());

            match self.build_cache.is_cached(&node)? {
                CacheHitType::Global => {
                    debug!("Skipping {}. Nothing to do.", name.to_string());
                    continue;
                }
                CacheHitType::Local => {
                    debug!("Skipping {}, but promoting outputs.", name.to_string());
                    self.build_cache
                        .promote_outputs(&node, &self.workspace.local_outputs_root)?;
                    continue;
                }
                CacheHitType::Miss => {
                    debug!("Cache miss! Proceeding to build...");
                }
            }

            let result = if node.target.is_local() {
                let mut sandbox = Sandbox::for_node(self.config.clone(), &self.workspace, &node);
                match sandbox.run(&self.build_cache)? {
                    ValidationStatus::Valid => {
                        self.build_cache.save(&sandbox)?;
                        sandbox.clear_sandbox()?;
                        targets += 1;
                        Ok(())
                    }
                    ValidationStatus::NoOutputs if node.outs().is_empty() => {
                        sandbox.clear_sandbox()?;
                        targets += 1;
                        Ok(())
                    }
                    ValidationStatus::NoOutputs => Err(anyhow!(
                        "Expected {} outputs, but found none.",
                        node.outs().len()
                    )),
                    ValidationStatus::Pending => Err(anyhow!(
                        "Node {} is somehow still pending...",
                        &name.to_string()
                    )),
                    ValidationStatus::Invalid {
                        expected_but_missing,
                        unexpected_but_present,
                        ..
                    } => Err(
                        anyhow!("Node {} expected the following but missing outputs: {:?}\n\ninstead it found the following unexpected outputs: {:?}",
                            &name.to_string(), expected_but_missing, unexpected_but_present)),
                }
            } else {
                debug!("Building global target...");
                node.execute(&self.config.archive_root, &self.config.cache_root)
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
}
