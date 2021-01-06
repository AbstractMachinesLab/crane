use super::Sandbox;
use anyhow::Context;
use log::debug;
use std::collections::HashMap;
use std::path::PathBuf;
use zap_core::{ComputedTarget, Label, ZapConfig};

/// The BuildCache implements an in-memory and persisted cache for build nodes
/// based on their hashes.
///
#[derive(Debug, Clone)]
pub struct BuildCache {
    root: PathBuf,
    memcache: HashMap<String, Label>,
}

impl BuildCache {
    pub fn new(config: &ZapConfig) -> BuildCache {
        BuildCache {
            root: config.cache_root.clone(),
            memcache: HashMap::<String, Label>::new(),
        }
    }

    pub fn save(&mut self, sandbox: &Sandbox) -> Result<(), anyhow::Error> {
        let node = sandbox.node();
        let hash = node.hash();
        self.memcache.insert(hash.clone(), node.label().clone());
        let cache_path = self.root.join(&hash);

        debug!(
            "Caching node {:?} hashed {:?}: {:?} outputs",
            node.label(),
            &hash,
            sandbox.outputs().len()
        );

        sandbox
            .outputs()
            .iter()
            .cloned()
            .map(|artifact| {
                debug!("Caching build artifact: {:?}", &artifact);
                let cached_file = cache_path.join(&artifact);

                // ensure all dirs are there for this file
                let cached_dir = &cached_file.parent().unwrap();
                debug!("Creating artifact cache path: {:?}", &cached_dir);
                std::fs::create_dir_all(&cached_dir)
                    .context(format!(
                        "Could not prepare directory for artifact {:?} into cache path: {:?}",
                        &artifact, &cached_dir
                    ))
                    .map(|_| ())?;

                let sandboxed_artifact = &sandbox.root().join(artifact);
                debug!(
                    "Moving artifact from sandbox path {:?} to cache path {:?}",
                    &sandboxed_artifact, &cached_file
                );
                std::fs::rename(&sandboxed_artifact, &cached_file)
                    .context(format!(
                        "Could not move artifact {:?} into cache path: {:?}",
                        sandboxed_artifact, cached_file
                    ))
                    .map(|_| ())?;

                Ok(())
            })
            .collect::<Result<(), anyhow::Error>>()
    }

    pub fn absolute_path_by_hash(&self, hash: &String) -> PathBuf {
        let path = self.root.join(hash);
        std::fs::canonicalize(&path).expect(&format!(
            "Could not find {:?} in disk, has the cache been modified manually?",
            &path
        ))
    }

    /// Determine if a given node has been cached already or not.
    ///
    /// This is based on hash of the node (see `BuildRule::hash`).
    ///
    /// FIXME: check if the expected hashes of the inputs match the actual
    /// hash of the files to determine if the cache is corrupted.
    pub fn is_cached(&mut self, node: &ComputedTarget) -> Result<bool, anyhow::Error> {
        let hash = node.hash();
        let cache_path = self.root.join(&hash);
        if std::fs::metadata(&cache_path).is_ok() {
            debug!("Cache hit for {:?} at {:?}", node.label(), cache_path);
            Ok(true)
        } else {
            debug!("No cache hit for {:?}", node.label());
            Ok(false)
        }
    }
}
