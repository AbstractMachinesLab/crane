use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Artifact {
    pub inputs: Vec<PathBuf>,
    pub outputs: Vec<PathBuf>,
}

impl Artifact {
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha1::new();

        let hashes = self.inputs
            .iter()
            .map(|path| {
                let contents = fs::read_to_string(&path)
                    .expect(&format!("Truly expected {:?} to be a readable file. Was it changed since the build started?", path));
                hasher.input_str(&contents);
                let hash = hasher.result_str();
                hasher.reset();
                hash
            }).collect::<Vec<String>>().join("");

        hasher.input_str(&hashes);
        hasher.result_str()
    }
}
