use super::{Action, Label, Target};
use anyhow::*;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use dashmap::DashMap;
use log::*;
use std::fs;
use std::path::PathBuf;
use zap_buildscript::*;

#[derive(Clone, Debug)]
pub struct Dependency {
    pub label: Label,
    pub hash: String,
    pub outs: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ComputedTarget {
    pub target: Target,

    /// The hash of this node
    hash: Option<String>,

    deps: Option<Vec<Dependency>>,

    /// The outputs of this node
    outs: Option<Vec<PathBuf>>,

    /// The inputs of this node
    srcs: Option<Vec<PathBuf>>,

    /// The actions to reify this target
    actions: Option<Vec<Action>>,

    /// Whether this ComputedTarget has already been successfully built, is waiting
    /// to be built, or has been found in the cache
    status: ComputeStatus,
}

#[derive(Debug, Clone)]
pub enum ComputeStatus {
    Uninitialized,
    Pending,
    CacheHit,
    Succeeded,
    Failed,
}

impl Default for ComputeStatus {
    fn default() -> ComputeStatus {
        ComputeStatus::Pending
    }
}

impl ComputedTarget {
    pub fn from_target(target: Target) -> ComputedTarget {
        ComputedTarget {
            target: target,
            status: ComputeStatus::Pending,
            actions: None,
            deps: None,
            hash: None,
            outs: None,
            srcs: None,
        }
    }

    pub fn as_dep(&self) -> Dependency {
        Dependency {
            label: self.target.label().clone(),
            hash: self.hash(),
            outs: self.outs(),
        }
    }

    pub fn mark_failed(&mut self) {
        self.status = ComputeStatus::Failed;
    }

    pub fn mark_succeeded(&mut self) {
        self.status = ComputeStatus::Succeeded;
    }

    pub fn hash(&self) -> String {
        self.hash.clone().unwrap_or_else(|| {
            panic!(
                "ComputedTarget {:?} has not been hashed yet!",
                self.label().to_string()
            )
        })
    }

    pub fn label(&self) -> &Label {
        self.target.label()
    }

    pub fn srcs(&self) -> Vec<PathBuf> {
        self.srcs.clone().unwrap_or_else(|| {
            panic!(
                "ComputedTarget {:?} does not have computed inputs yet!",
                self.label().to_string()
            )
        })
    }

    pub fn outs(&self) -> Vec<PathBuf> {
        self.outs.clone().unwrap_or_else(|| {
            panic!(
                "ComputedTarget {:?} does not have computed outputs yet!",
                self.label().to_string()
            )
        })
    }

    pub fn deps(&self) -> Vec<Dependency> {
        self.deps.clone().unwrap_or_else(|| {
            panic!(
                "ComputedTarget {:?} does not have computed dependencies yet!",
                self.label().to_string()
            )
        })
    }

    pub fn actions(&self) -> Vec<Action> {
        self.actions.clone().unwrap_or_else(|| {
            panic!(
                "ComputedTarget {:?} does not have computed actions yet!",
                self.label().to_string()
            )
        })
    }

    pub fn execute(&self) -> Result<(), anyhow::Error> {
        for action in self.actions() {
            action.run()?
        }
        Ok(())
    }

    pub fn seal(
        &mut self,
        deps: &[Dependency],
        action_map: &DashMap<Label, Vec<Action>>,
        output_map: &DashMap<Label, Vec<PathBuf>>,
        bs_ctx: &mut BuildScript,
    ) -> Result<(), anyhow::Error> {
        let label = self.target.label().clone();
        trace!("Sealing Computed Target {:?}", label.to_string());

        let config: serde_json::Value = self.target.config().clone().into();

        let transitive_deps: serde_json::Value = serde_json::Value::Array(
            deps.iter()
                .map(|dep| {
                    let mut map = serde_json::Map::new();
                    map.insert(
                        "label".to_string(),
                        serde_json::Value::String(dep.label.to_string()),
                    );
                    map.insert(
                        "outs".to_string(),
                        serde_json::Value::Array(
                            dep.outs
                                .iter()
                                .map(|p| serde_json::Value::String(p.to_str().unwrap().to_string()))
                                .collect(),
                        ),
                    );
                    serde_json::Value::Object(map)
                })
                .collect(),
        );
        let compute_program = include_str!("compute_target.js")
            .replace("{LABEL_NAME}", &label.to_string())
            .replace("{RULE_NAME}", self.target.rule().name())
            .replace("{CONFIG}", &config.to_string())
            .replace("{TRANSITIVE_DEPS}", &transitive_deps.to_string());

        trace!("Executing: {}", &compute_program);

        bs_ctx.runtime.execute(
            &format!("<computed_target: {:?}>", &label.to_string()),
            &compute_program,
        )?;

        let actions = action_map
            .get(&label)
            .map(|entry| entry.value().clone())
            .unwrap_or(vec![]);

        let outs = output_map
            .get(&label)
            .context(format!(
                "Could not find declared outputs for target  {:?}  - ",
                &label.to_string()
            ))?
            .clone();

        let srcs = self.target.config().get_file_list("srcs").unwrap_or(vec![]);

        self.deps = Some(deps.to_vec());
        self.srcs = Some(srcs);
        self.outs = Some(outs);
        self.actions = Some(actions);

        self.update_hash();

        trace!(
            "Sealed ComputedTarget {} with Hash {:?}",
            label.to_string(),
            self.hash.as_ref().unwrap()
        );

        Ok(())
    }

    /// The hash of a build node serves for caching work:
    /// * the hash of the dependencies
    /// * listed inputs, and their contents
    /// * listed outputs
    /// * rule name
    /// * the hash of the computed actions that this target will execute
    ///
    fn update_hash(&mut self) {
        let mut hasher = Sha1::new();

        let name = self.target.label();
        hasher.input_str(&name.to_string());

        for d in self.deps.as_ref().unwrap() {
            hasher.input_str(&d.hash);
        }

        for src_path in self.srcs.as_ref().unwrap() {
            let contents = fs::read_to_string(&src_path)
                .unwrap_or_else(|_| panic!("Truly expected {:?} to be a readable file. Was it changed since the build started?", src_path));
            hasher.input_str(&contents);
        }

        for o in self.outs.as_ref().unwrap() {
            hasher.input_str(o.to_str().unwrap());
        }

        for a in self.actions.as_ref().unwrap() {
            // TODO(@ostera): implement Hash for Action
            hasher.input_str(&format!("{:?}", a));
        }

        let hash = hasher.result_str();
        self.hash = Some(hash);
    }
}

impl Default for ComputedTarget {
    fn default() -> Self {
        panic!("Oops! We have attempted to create a default target. This means the DepGraph was somehow incomplete. This is a bug!");
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rule;
    use std::path::PathBuf;

    fn toolchain_mgr() -> ToolchainManager {
        ToolchainManager::new()
    }

    fn unsealed_target() -> ComputedTarget {
        ComputedTarget::from_target(Box::new(TestTarget::new()))
    }

    fn sealed_target() -> ComputedTarget {
        let mut target = ComputedTarget::from_target(Box::new(TestTarget::new()));
        target.seal(&vec![], &toolchain_mgr()).unwrap();
        target
    }

    #[test]
    #[should_panic]
    fn must_be_sealed_to_access_srcs() {
        let target = unsealed_target();
        target.srcs();
    }

    #[test]
    #[should_panic]
    fn must_be_sealed_to_access_outs() {
        let target = unsealed_target();
        target.outs();
    }

    #[test]
    #[should_panic]
    fn must_be_sealed_to_access_deps() {
        let target = unsealed_target();
        target.deps();
    }

    #[test]
    #[should_panic]
    fn must_be_sealed_to_access_hash() {
        let target = unsealed_target();
        target.hash();
    }

    #[test]
    fn can_access_srcs_after_sealing() {
        let target = sealed_target();
        target.srcs();
    }

    #[test]
    fn can_access_outs_after_sealing() {
        let target = sealed_target();
        target.outs();
    }

    #[test]
    fn can_access_deps_after_sealing() {
        let target = sealed_target();
        target.deps();
    }

    #[test]
    fn can_access_hash_after_sealing() {
        let target = sealed_target();
        target.hash();
    }

    #[test]
    fn srcs_are_part_of_hashing() {
        let a = sealed_target();

        let mut b = TestTarget::new();
        b.set_srcs(&vec![PathBuf::from("src")]);
        let mut b = ComputedTarget::from_target(Box::new(b));
        b.seal(&vec![], &toolchain_mgr()).unwrap();

        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn deps_are_part_of_hashing() {
        let a = sealed_target();

        let mut b = ComputedTarget::from_target(Box::new(TestTarget::new()));
        b.seal(
            &vec![Dependency {
                label: Label::new(":a"),
                hash: "some-hash".to_string(),
                outs: vec![],
            }],
            &toolchain_mgr(),
        )
        .unwrap();

        assert_ne!(a.hash, b.hash);
    }
}
*/
