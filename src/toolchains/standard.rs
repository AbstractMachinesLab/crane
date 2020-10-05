use flate2::write::GzEncoder;
use flate2::Compression;
use log::debug;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Toolchain {}

impl Default for Toolchain {
    fn default() -> Toolchain {
        Toolchain {}
    }
}

impl Toolchain {
    pub fn tar(self, archive_name: &PathBuf, paths: &[PathBuf]) -> Result<(), anyhow::Error> {
        debug!("Creating Archive {:?} with: {:?}", &archive_name, &paths);
        let archive = File::create(archive_name)?;
        let encoder = GzEncoder::new(archive, Compression::default());
        let mut builder = tar::Builder::new(encoder);
        for path in paths {
            builder.append_path(path)?;
        }
        Ok(())
    }
}
