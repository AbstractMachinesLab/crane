use super::{Artifact, BuildRule};
use crate::label::Label;
use crypto::digest::Digest;
use crypto::sha1::Sha1;

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
        self.hash.clone().unwrap_or("not-yet-hashed".to_string())
    }

    pub fn name(&self) -> Label {
        self.rule.name().clone()
    }

    pub fn outs(&self) -> Vec<Artifact> {
        self.rule.outputs()
    }

    pub fn deps(&self) -> Vec<Label> {
        self.rule.dependencies()
    }

    pub fn rule(&mut self) -> &mut BuildRule {
        &mut self.rule
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
        for d in deps {
            hasher.input_str(&d.hash());
        }
        for o in self.rule.outputs() {
            hasher.input_str(&o.compute_hash(name.path()));
        }

        let hash = hasher.result_str();

        self.hash = Some(hash);
    }
}
