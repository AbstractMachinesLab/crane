use crane_core::{Artifact, Label, Target};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::path::PathBuf;

/// A Build Node represents a compilation unit in the Build Graph.
///
/// It is associated with a label, it has a hash that can be used to find it
/// in the BuildCache, and it is linked to a Rule that defines how exactly
/// this unit of work should be completed.
#[derive(Debug)]
pub struct BuildNode {
    target: Box<dyn Target>,

    /// The hash of this node
    hash: Option<String>,

    /// The outputs of this node
    outs: Option<Vec<Artifact>>,

    /// The inputs of this node
    srcs: Option<Vec<PathBuf>>,

    /// Whether this BuildNode has already been successfully built, is waiting
    /// to be built, or has been found in the cache
    status: NodeStatus,
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
    pub fn from_target(target: Box<dyn Target>) -> BuildNode {
        BuildNode {
            target: target,
            status: NodeStatus::Pending,
            hash: None,
            outs: None,
            srcs: None,
        }
    }

    pub fn mark_failed(&mut self) {
        self.status = NodeStatus::Failed;
    }

    pub fn mark_succeeded(&mut self) {
        self.status = NodeStatus::Succeeded;
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
        self.target.name().clone()
    }

    pub fn srcs(&self) -> Vec<PathBuf> {
        self.srcs.clone().unwrap_or_else(|| {
            panic!(
                "Node {:?} does not have computed inputs yet!",
                self.name().to_string()
            )
        })
    }

    pub fn outs(&self) -> Vec<Artifact> {
        self.outs.clone().unwrap_or_else(|| {
            panic!(
                "Node {:?} does not have computed outputs yet!",
                self.name().to_string()
            )
        })
    }

    pub fn deps(&self) -> &[Label] {
        self.target.deps()
    }

    pub fn execute(&self) -> Result<(), anyhow::Error> {
        self.target.rule().execute()
    }

    /// The hash of a build node serves for caching work:
    /// * the hash of the dependencies
    /// * listed inputs, and their contents
    /// * listed outputs
    /// * rule name
    ///
    /// FIXME(@ostera): add a pluggable way to add relevant information to this hash to
    /// use build options in it
    pub fn update_hash(&mut self, deps: Vec<(String, Vec<Artifact>)>) {
        let mut hasher = Sha1::new();
        let name = self.target.name();
        hasher.input_str(&name.to_string());

        let dep_outs: Vec<Artifact> = deps
            .iter()
            .flat_map(|(hash, outs)| {
                hasher.input_str(&hash);
                outs
            })
            .cloned()
            .collect();

        let srcs = self.target.srcs(&dep_outs);
        self.srcs = Some(srcs.to_vec());
        for i in srcs {
            hasher.input_str(&i.to_str().unwrap())
        }

        let outs = self.target.outputs(&dep_outs);
        self.outs = Some(outs.to_vec());
        for o in outs {
            let hashed_output = o.hash_with_prefix(name.path());
            hasher.input_str(hashed_output.hash());
        }

        let hash = hasher.result_str();
        self.hash = Some(hash);
    }
}
