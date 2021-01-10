use anyhow::Context;
use log::*;
use regex::Regex;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileScanner {
    root: PathBuf,
    match_pattern: Regex,
}

impl FileScanner {
    pub fn new() -> FileScanner {
        FileScanner {
            root: PathBuf::from("."),
            match_pattern: Regex::new(".*").unwrap(),
        }
    }

    pub fn starting_from(&mut self, root: PathBuf) -> &mut FileScanner {
        self.root = root.clone();
        self
    }

    pub fn matching_path(&mut self, pattern: &str) -> Result<&mut FileScanner, anyhow::Error> {
        self.match_pattern = Regex::new(pattern)?;
        Ok(self)
    }

    pub fn find_files(&self) -> Result<Vec<PathBuf>, anyhow::Error> {
        self.find_files_aux(self.root.clone())
    }

    fn find_files_aux(&self, current: PathBuf) -> Result<Vec<PathBuf>, anyhow::Error> {
        if current.is_dir() {
            trace!("Reading dir {:?}", current);
            Ok(fs::read_dir(current)?
                .flat_map(|entry| {
                    let path = entry?.path();
                    if path.is_dir() {
                        self.find_files_aux(path)
                    } else if self.match_pattern.is_match(path.to_str().context(format!("Could not run pattern {:?} on path {:?} since it is not a valid UTF-8 string", self.match_pattern, path))?) {
                        Ok(vec![path])
                    } else {
                        Ok(vec![])
                    }
                })
                .flatten()
                .collect())
        } else {
            trace!("Skipping path {:?}", current);
            Ok(vec![])
        }
    }
}
