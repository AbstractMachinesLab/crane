use anyhow::anyhow;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::fs;
use std::path::PathBuf;

/// An artifact in Crane, described by its inputs and outputs.
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

    pub fn hash_with_prefix(self, path_prefix: PathBuf) -> HashedArtifact {
        HashedArtifact::new(self, path_prefix)
    }
}

/// A hashed artifact in Crane.
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
    pub fn set_inputs(&mut self, inputs: &[PathBuf]) -> &mut ArtifactBuilder {
        self.inputs = Some(inputs.to_vec());
        self
    }

    pub fn set_outputs(&mut self, outputs: &[PathBuf]) -> &mut ArtifactBuilder {
        self.outputs = Some(outputs.to_vec());
        self
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
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
