/*
use anyhow::anyhow;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::fs;
use std::path::PathBuf;

/// An artifact in Zap, described by its inputs and outputs.
///
#[derive(Debug, Clone)]
pub struct Artifact {
    inputs: Vec<PathBuf>,
    outputs: Vec<PathBuf>,
}

impl Artifact {
    pub fn builder() -> ArtifactBuilder {
        ArtifactBuilder::default()
    }

    pub fn inputs(&self) -> &[PathBuf] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[PathBuf] {
        &self.outputs
    }

    pub fn hash_with_prefix(&self, path_prefix: PathBuf) -> HashedArtifact {
        HashedArtifact::new(self.clone(), path_prefix)
    }
}

/// A hashed artifact in Zap.
///
/// This is normally an artifact that has been hashed using a specific path prefix
/// to know where to read the files from.
///
#[derive(Debug, Clone)]
pub struct HashedArtifact {
    inputs: Vec<PathBuf>,
    outputs: Vec<PathBuf>,
    prefix: PathBuf,
    hash: String,
}

impl HashedArtifact {
    fn new(artifact: Artifact, prefix: PathBuf) -> HashedArtifact {
        let inputs = artifact.inputs;
        let outputs = artifact.outputs;
        let hash = {
            let mut hasher = Sha1::new();
            hasher.input_str(prefix.to_str().unwrap());
            for path in inputs.iter() {
                let path = prefix.join(path);
                let contents = fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("Truly expected {:?} to be a readable file. Was it changed since the build started?", path));
                hasher.input_str(&contents);
            }
            for path in outputs.iter() {
                hasher.input_str(path.to_str().unwrap());
            }
            hasher.result_str()
        };
        HashedArtifact {
            inputs,
            outputs,
            prefix,
            hash,
        }
    }

    pub fn hash(&self) -> &String {
        &self.hash
    }
}

#[derive(Debug, Clone, Default)]
pub struct ArtifactBuilder {
    inputs: Option<Vec<PathBuf>>,
    outputs: Option<Vec<PathBuf>>,
}

impl ArtifactBuilder {
    pub fn set_inputs(self, inputs: &[PathBuf]) -> ArtifactBuilder {
        let inputs = Some(inputs.to_vec());
        ArtifactBuilder { inputs, ..self }
    }

    pub fn set_outputs(self, outputs: &[PathBuf]) -> ArtifactBuilder {
        let outputs = Some(outputs.to_vec());
        ArtifactBuilder { outputs, ..self }
    }

    pub fn build(self) -> Result<Artifact, anyhow::Error> {
        self.assert_complete()?;
        let inputs = self.inputs.unwrap();
        let outputs = self.outputs.unwrap();
        Ok(Artifact { inputs, outputs })
    }

    fn assert_complete(&self) -> Result<(), anyhow::Error> {
        if self.inputs.is_none() {
            return Err(anyhow!("Attempted to build artifact without inputs."));
        };
        if self.outputs.is_none() {
            return Err(anyhow!("Attempted to build artifact without outputs."));
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_hash_an_empty_artifact() {
        let artifact = Artifact::builder()
            .set_inputs(&vec![])
            .set_outputs(&vec![])
            .build()
            .unwrap();
        let hashed_artifact = artifact.hash_with_prefix(PathBuf::from(""));
        let hash = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
        assert_eq!(hash, hashed_artifact.hash());
    }

    #[test]
    fn prefix_is_part_of_hash() {
        let artifact = Artifact::builder()
            .set_inputs(&vec![])
            .set_outputs(&vec![])
            .build()
            .unwrap();
        let hashed_artifact_0 = artifact.hash_with_prefix(PathBuf::from("."));
        let hashed_artifact_1 = artifact.hash_with_prefix(PathBuf::from("./some/path"));
        assert_eq!(
            "3a52ce780950d4d969792a2559cd519d7ee8c727",
            hashed_artifact_0.hash()
        );
        assert_eq!(
            "0e1be165e3e4a885eb846d37d46fdcf56fa7372d",
            hashed_artifact_1.hash()
        );
    }
}
*/
