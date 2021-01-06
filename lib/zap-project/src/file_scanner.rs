use anyhow::Context;
use log::*;
use regex::Regex;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct FileScanner {}

impl FileScanner {
    pub fn new() -> FileScanner {
        FileScanner {}
    }

    pub fn find_with_pattern(
        &self,
        root: &PathBuf,
        pattern: &str,
    ) -> Result<Vec<PathBuf>, anyhow::Error> {
        self.find_files(root, &root.clone(), &Regex::new(pattern)?)
    }

    fn find_files(
        &self,
        root: &PathBuf,
        current: &PathBuf,
        pattern: &Regex,
    ) -> Result<Vec<PathBuf>, anyhow::Error> {
        if current.is_dir() {
            trace!("Reading dir {:?}", current);
            Ok(fs::read_dir(current)?
                .flat_map(|entry| {
                    let path = entry?.path();
                    if path.is_dir() {
                        self.find_files(&root, &path, &pattern)
                    } else if pattern.is_match(path.to_str().context(format!("Could not run pattern {:?} on path {:?} since it is not a valid UTF-8 string", pattern, path))?) {
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
