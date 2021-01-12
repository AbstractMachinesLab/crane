use super::{parsers, Buildfile, RuleManager, ToolchainManager, ZAPFILE};
use anyhow::Context;
use log::*;
use std::fs;
use std::path::PathBuf;
use super::{Workspace, WORKSPACE};

pub struct WorkspaceScanner {}

impl WorkspaceScanner {
    pub fn scan(
        root: &PathBuf,
        toolchain_manager: &ToolchainManager,
    ) -> Result<Workspace, anyhow::Error> {
        let cwd = fs::canonicalize(&root)?;
        debug!("Scanning workspace at: {:?}", cwd);

        let workspace_file = WorkspaceScanner::find_workspace_file_upwards(&cwd)?;
        let workspace_root = &workspace_file
            .parent()
            .context("Could not get parent dir for the workspace file path")?
            .to_path_buf();
        WorkspaceScanner::from_toml_file(workspace_file, &workspace_root, &toolchain_manager)
    }

    pub fn collect_targets<'a>(
        workspace: &'a mut Workspace,
        rule_manager: &RuleManager,
    ) -> Result<&'a mut Workspace, anyhow::Error> {
        let paths = WorkspaceScanner::find_files(workspace.root(), workspace.root());
        debug!("Found {} build files...", paths.len());

        let mut targets = vec![];
        for path in paths {
            let buildfile = Buildfile::from_file(&workspace.root(), &path, &rule_manager)?;
            for target in buildfile.targets() {
                targets.push(target);
            }
        }
        debug!("Found {} build targets...", targets.len());

        Ok(workspace.with_targets(targets))
    }

    pub fn from_toml_file(
        path: PathBuf,
        root: &PathBuf,
        toolchain_manager: &ToolchainManager,
    ) -> Result<Workspace, anyhow::Error> {
        let toml = fs::read_to_string(&path)
            .context(format!("Could not read file {:?}", &path))?
            .parse::<toml::Value>()
            .context(format!("Could not parse file {:?} as TOML", &path))?;

        debug!("Loading workspace file from {:?}", &path);
        parsers::workspace::parse(toml, root, toolchain_manager)
    }

    fn find_workspace_file_upwards(cwd: &PathBuf) -> Result<PathBuf, anyhow::Error> {
        let here = &cwd.join(WORKSPACE);
        debug!("Searching for workspace file in {:?}", here);
        if fs::metadata(here).is_ok() {
            debug!("Found it! Continuing...");
            Ok(here.to_path_buf())
        } else {
            let parent = cwd.parent().context(
                "Reached the top of the file system and could not find a workspace file",
            )?;
            WorkspaceScanner::find_workspace_file_upwards(&parent.to_path_buf())
        }
    }

    fn find_files(root: &PathBuf, current: &PathBuf) -> Vec<PathBuf> {
        if current.is_dir() {
            trace!("Reading dir {:?}", current);
            fs::read_dir(current)
                .unwrap_or_else(|err| {
                    panic!("Could not read directory: {:?} due to {:?}", current, err)
                })
                .flat_map(|entry| {
                    let entry = entry.unwrap_or_else(|_| panic!("Could not read entry"));
                    let path = entry.path();

                    if path.is_dir() {
                        WorkspaceScanner::find_files(&root, &path)
                    } else {
                        let name = path.file_name().unwrap().to_str().unwrap();
                        trace!("Reading file {:?}", name);
                        if name.eq_ignore_ascii_case(ZAPFILE) {
                            vec![path]
                        } else {
                            vec![]
                        }
                    }
                })
                .collect()
        } else {
            trace!("Skipping path {:?}", current);
            vec![]
        }
    }
}
