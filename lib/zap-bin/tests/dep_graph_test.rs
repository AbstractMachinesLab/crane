use zap_core::*;
use zap_project::*;
use std::path::PathBuf;

#[test]
fn it_reads_the_project_from_an_absolute_path() {
    let root = std::fs::canonicalize(PathBuf::from(&"./tests/sample_project")).unwrap();
    let workspace = WorkspaceScanner::scan(&root).unwrap();
    assert_eq!("sample_project", workspace.name());
    assert_eq!(root.to_str(), workspace.root().to_str());
    assert_eq!(3, workspace.targets().len());
}

#[test]
fn it_reads_the_project_from_a_relative_path() {
    let root = PathBuf::from(&"./tests/sample_project");
    let canonical_root = std::fs::canonicalize(&root).unwrap();
    let workspace = WorkspaceScanner::scan(&root).unwrap();
    assert_eq!("sample_project", workspace.name());
    assert_eq!(canonical_root.to_str(), workspace.root().to_str());
    assert_eq!(3, workspace.targets().len());
}

#[test]
fn build_dependency_graph_from_workspace() {
    let root = PathBuf::from(&"./tests/sample_project");
    let workspace = WorkspaceScanner::scan(&root).unwrap();

    let mut dep_graph = DepGraph::from_targets(&workspace.targets()).unwrap();

    let target_names_in_order = dep_graph.target_names();
    assert_eq!(3, target_names_in_order.len());
    assert_eq!(
        r#"["//a:lib", "//b/c:lib", "//b:lib"]"#,
        format!("{:?}", target_names_in_order)
    );
}

#[test]
fn build_dependency_graph_from_workspace_and_scope() {
    let root = PathBuf::from(&"./tests/sample_project");
    let workspace = WorkspaceScanner::scan(&root).unwrap();

    let mut dep_graph = DepGraph::from_targets(&workspace.targets())
        .unwrap()
        .scoped(Label::new("//b/c:lib"))
        .unwrap();

    let target_names_in_order = dep_graph.target_names();
    assert_eq!(2, target_names_in_order.len());
    assert_eq!(
        r#"["//a:lib", "//b/c:lib"]"#,
        format!("{:?}", target_names_in_order)
    );
}

#[test]
fn configure_toolchain_manager_when_reading_workspace() {
    let root = PathBuf::from(&"./tests/sample_project");
    let workspace = WorkspaceScanner::scan(&root).unwrap();
    assert_eq!(
        1,
        workspace.toolchain_manager().toolchains_as_targets().len()
    );
}
