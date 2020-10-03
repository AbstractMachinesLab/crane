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
    pub fn compute_hash(&self, path_prefix: PathBuf) -> String {
        let mut hasher = Sha1::new();
        for path in self.inputs.iter() {
            let path = path_prefix.join(path);
            let contents = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Truly expected {:?} to be a readable file. Was it changed since the build started?", path));
            hasher.input_str(&contents);
        }
        for path in self.outputs.iter() {
            hasher.input_str(path.to_str().unwrap());
        }
        hasher.result_str()
    }
}
