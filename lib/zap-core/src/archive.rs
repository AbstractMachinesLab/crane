use anyhow::{anyhow, Context};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use log::{debug, info};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub enum ArchiveKind {
    Source,
    Release,
}

impl Default for ArchiveKind {
    fn default() -> Self {
        ArchiveKind::Source
    }
}

#[derive(Debug, Clone, Default)]
pub struct Archive {
    /// The path to the unarchived contents
    unarchived_root: PathBuf,

    kind: ArchiveKind,
    url: String,
    sha1: String,
    name: String,
    tag: String,
    prefix: String,
}

impl Archive {
    pub fn new() -> Archive {
        Archive::default()
    }

    pub fn hash(&self) -> String {
        let mut hasher = Sha1::new();
        let archive = format!("{}:{}", &self.url, &self.sha1);
        hasher.input_str(&archive.as_str());
        hasher.result_str()
    }

    pub fn url(&self) -> String {
        match self.kind {
            ArchiveKind::Source => self.url.clone(),
            ArchiveKind::Release => {
                let host_triple = guess_host_triple::guess_host_triple().unwrap();
                format!(
                    "{prefix}/releases/download/{tag}/{name}-{host_triple}.{ext}",
                    prefix = self.url.clone(),
                    tag = self.tag.clone(),
                    name = self.name.clone(),
                    host_triple = host_triple,
                    ext = "tar.gz",
                )
            }
        }
    }

    pub fn unarchived_root(&self) -> &PathBuf {
        &self.unarchived_root
    }

    pub fn sha1(&self) -> String {
        self.sha1.clone()
    }

    pub fn prefix(&self) -> String {
        self.prefix.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn file_name(&self) -> String {
        "toolchain.tar.gz".to_string()
    }

    pub fn with_url(self, url: String) -> Archive {
        Archive { url, ..self }
    }

    pub fn with_tag(self, tag: String) -> Archive {
        Archive { tag, ..self }
    }

    pub fn with_sha1(self, sha1: String) -> Archive {
        Archive { sha1, ..self }
    }

    pub fn with_prefix(self, prefix: String) -> Archive {
        Archive { prefix, ..self }
    }

    pub fn with_name(self, name: String) -> Archive {
        Archive { name, ..self }
    }

    pub fn as_source(self) -> Archive {
        Archive {
            kind: ArchiveKind::Source,
            ..self
        }
    }

    pub fn as_release(self) -> Archive {
        Archive {
            kind: ArchiveKind::Release,
            ..self
        }
    }

    pub fn validate(&self) {
        match self.kind {
            ArchiveKind::Release => {
                if self.name.is_empty() {
                    panic!("Release Archives MUST specify a name attribute!")
                }
            }
            _ => (),
        }
    }

    pub fn is_cached(&self, outdir: &PathBuf) -> Result<bool, anyhow::Error> {
        let path = outdir.join(self.file_name());
        let result = std::fs::metadata(&path).is_ok();
        debug!("Checking if toolchain exists in {:?}: {:?}", path, result);
        Ok(result)
    }

    pub fn checksum(&self, outdir: &PathBuf) -> Result<bool, anyhow::Error> {
        let mut hasher = Sha1::new();
        let archive = &outdir.join(&self.file_name());
        debug!(
            "Checking if archive at: {:?} has checksum {:?}",
            &archive, &self.sha1
        );
        let mut file = std::fs::File::open(&archive).context(format!(
            "Truly expected {:?} to be a readable file. Was it changed since the build started?",
            archive
        ))?;
        let mut contents: Vec<u8> = std::vec::Vec::with_capacity(file.metadata()?.len() as usize);
        file.read_to_end(&mut contents)?;
        hasher.input(&contents);
        let hash = hasher.result_str();
        if hash == self.sha1 {
            Ok(true)
        } else {
            Err(anyhow!(
                "Expected archive to have SHA1 {} but found {} instead",
                self.sha1,
                hash
            ))
        }
    }

    pub fn download(&self, outdir: &PathBuf) -> Result<(), anyhow::Error> {
        info!(
            "Downloading toolchain from {:?} into {:?}",
            self.url(),
            &outdir
        );

        std::fs::create_dir_all(&outdir).context(format!(
            "Failed to create toolchain root folder at {:?}",
            &outdir
        ))?;

        let wget = Command::new("wget")
            .args(&[self.url()])
            .args(&["-O", self.file_name().as_str()])
            .current_dir(&outdir)
            .output()
            .context("Could not run wget")?;

        if wget.status.success() {
            Ok(())
        } else {
            std::io::stdout().write_all(&wget.stdout).unwrap();
            std::io::stderr().write_all(&wget.stderr).unwrap();
            Err(anyhow!("Error downloading toolchain!"))
        }
    }

    pub fn unpack(&self, final_dir: &PathBuf) -> Result<(), anyhow::Error> {
        let tar = Command::new("tar")
            .args(&["xzf", "toolchain.tar.gz"])
            .current_dir(&final_dir)
            .output()
            .context("Could not run tar")?;

        if tar.status.success() {
            Ok(())
        } else {
            std::io::stdout().write_all(&tar.stdout).unwrap();
            std::io::stderr().write_all(&tar.stderr).unwrap();
            Err(anyhow!("Error downloading toolchain!"))
        }
    }
}