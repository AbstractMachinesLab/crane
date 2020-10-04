use super::{Artifact, BuildPlan, BuildRule};
use crate::label::Label;
use crate::toolchains::Toolchains;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::path::PathBuf;

/// A Build Node represents a compilation unit in the Build Graph.
///
/// It is associated with a label, it has a hash that can be used to find it
/// in the BuildCache, and it is linked to a BuildRule that defines how exactly
/// this unit of work should be completed.
#[derive(Debug, Clone, Default)]
pub struct BuildNode {
    /// The name of the build node
    name: Label,

    /// The hash of this node
    hash: Option<String>,

    /// The outputs of this node
    outputs: Option<Vec<Artifact>>,

    /// The inputs of this node
    inputs: Option<Vec<PathBuf>>,

    /// Whether this BuildNode has already been successfully built, is waiting
    /// to be built, or has been found in the cache
    status: NodeStatus,

    /// The rule defining how to complete this work
    rule: BuildRule,
}

#[derive(Debug, Clone)]
pub enum NodeStatus {
    Pending,
    CacheHit,
    Succeeded,
    Failed,
}

impl Default for NodeStatus {
    fn default() -> NodeStatus {
        NodeStatus::Pending
    }
}

impl BuildNode {
    pub fn from_rule(rule: BuildRule) -> BuildNode {
        let name = rule.name();
        BuildNode {
            name,
            rule,
            status: NodeStatus::Pending,
            hash: None,
            outputs: None,
            inputs: None,
        }
    }

    pub fn mark_failed(self) -> BuildNode {
        BuildNode {
            status: NodeStatus::Failed,
            ..self
        }
    }

    pub fn mark_succeeded(self) -> BuildNode {
        BuildNode {
            status: NodeStatus::Succeeded,
            ..self
        }
    }

    pub fn hash(&self) -> String {
        self.hash.clone().unwrap_or_else(|| {
            panic!(
                "Node {:?} has not been hashed yet!",
                self.name().to_string()
            )
        })
    }

    pub fn name(&self) -> Label {
        self.rule.name().clone()
    }

    pub fn srcs(&self) -> Vec<PathBuf> {
        self.inputs.clone().unwrap_or_else(|| {
            panic!(
                "Node {:?} does not have computed inputs yet!",
                self.name().to_string()
            )
        })
    }

    pub fn outs(&self) -> Vec<Artifact> {
        self.outputs.clone().unwrap_or_else(|| {
            panic!(
                "Node {:?} does not have computed outputs yet!",
                self.name().to_string()
            )
        })
    }

    pub fn deps(&self) -> Vec<Label> {
        self.rule.dependencies()
    }

    pub fn rule(&mut self) -> &mut BuildRule {
        &mut self.rule
    }

    pub fn build(&mut self, plan: &BuildPlan, toolchain: &Toolchains) -> Result<(), anyhow::Error> {
        let inputs = self.srcs();
        self.rule
            .clone()
            .set_inputs(inputs)
            .build(&plan, &toolchain)
    }

    /// The hash of a build node serves for caching work:
    /// * the hash of the dependencies
    /// * listed inputs, and their contents
    /// * listed outputs
    /// * rule name
    ///
    /// FIXME(@ostera): add a pluggable way to add relevant information to this hash to
    /// use build options in it
    pub fn update_hash(&mut self, deps: Vec<BuildNode>) {
        let mut hasher = Sha1::new();
        let name = self.rule.name();
        hasher.input_str(&name.to_string());

        let dep_outs: Vec<Artifact> = deps
            .iter()
            .flat_map(|d| {
                hasher.input_str(&d.hash());
                d.outs()
            })
            .collect();

        self.inputs = Some(self.rule.inputs(&dep_outs));
        for i in self.inputs.clone().unwrap_or(vec![]) {
            hasher.input_str(&i.to_str().unwrap())
        }

        self.outputs = Some(self.rule.outputs(&dep_outs));
        for o in self.outputs.clone().unwrap_or(vec![]) {
            hasher.input_str(&o.compute_hash(name.path()));
        }

        let hash = hasher.result_str();
        self.hash = Some(hash);
    }
}
