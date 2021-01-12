use super::*;
use log::*;
use std::fs;
use std::path::PathBuf;

pub struct RuleScanner {}

impl RuleScanner {
    pub fn scan(root: &PathBuf) -> Result<Vec<PathBuf>, anyhow::Error> {
        let root = fs::canonicalize(&root)?;
        debug!("Scanning for rules in {:?}", root);
        let mut fs = FileScanner::new();

        fs.starting_from(root).matching_path("\\.js$")?;

        fs.find_files()
    }
}
