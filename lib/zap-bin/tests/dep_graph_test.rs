use std::path::PathBuf;
use zap_core::*;
use zap_project::*;

#[tokio::test]
async fn it_reads_the_project_from_an_absolute_path() {
    let root = std::fs::canonicalize(PathBuf::from(&"./tests/sample_project")).unwrap();

    let config = ZapConfig::new().unwrap();
    let mut zap = ZapWorker::new(config).unwrap();
    zap.load(&root).await.unwrap();

    let workspace = zap.workspace;
    assert_eq!("sample_project", workspace.name());
    assert_eq!(root.to_str(), workspace.root().to_str());
    assert_eq!(4, workspace.targets().len());
}

#[tokio::test]
async fn it_reads_the_project_from_a_relative_path() {
    let root = PathBuf::from(&"./tests/sample_project");

    let config = ZapConfig::new().unwrap();
    let mut zap = ZapWorker::new(config).unwrap();
    zap.load(&root).await.unwrap();

    let workspace = zap.workspace;
    assert_eq!("sample_project", workspace.name());
    assert_eq!(
        std::fs::canonicalize(root).unwrap().to_str(),
        workspace.root().to_str()
    );
    assert_eq!(4, workspace.targets().len());
}

#[tokio::test]
async fn build_dependency_graph_from_workspace() {
    let root = PathBuf::from(&"./tests/sample_project");

    let config = ZapConfig::new().unwrap();
    let mut zap = ZapWorker::new(config).unwrap();
    zap.load(&root).await.unwrap();

    let mut dep_graph = zap.dep_graph;

    let mut target_names_in_order = dep_graph.target_names();
    // NOTE(@ostera): we alphabetically to be able to assert deterministically
    target_names_in_order.sort();

    let mut expected = vec![
        format!(
            "{}:{}",
            PathBuf::from("//").join("a").to_str().unwrap(),
            "lib"
        ),
        format!(
            "{}:{}",
            PathBuf::from("//").join("b").join("c").to_str().unwrap(),
            "lib"
        ),
        format!(
            "{}:{}",
            PathBuf::from("//").join("b").join("c").to_str().unwrap(),
            "my_archive"
        ),
        format!(
            "{}:{}",
            PathBuf::from("//").join("b").to_str().unwrap(),
            "lib"
        ),
        ":caramel".to_string(),
        ":erlang".to_string(),
    ];
    expected.sort();

    assert_eq!(6, target_names_in_order.len());
    assert_eq!(
        format!("{:?}", expected),
        format!("{:?}", target_names_in_order)
    );
}

#[tokio::test]
async fn build_dependency_graph_from_workspace_and_scope() {
    let root = PathBuf::from(&"./tests/sample_project");

    let config = ZapConfig::new().unwrap();
    let mut zap = ZapWorker::new(config).unwrap();
    zap.load(&root).await.unwrap();

    let dep_graph = zap.dep_graph.scoped(&Label::new("//b/c:lib")).unwrap();

    let target_names_in_order = dep_graph.target_names();
    assert_eq!(4, target_names_in_order.len());
    assert_eq!(
        r#"[":erlang", "//a:lib", "//b:lib", "//b/c:lib"]"#,
        format!("{:?}", target_names_in_order)
    );
}

#[tokio::test]
async fn configure_toolchain_manager_when_reading_workspace() {
    let root = std::fs::canonicalize(PathBuf::from(&"./tests/sample_project")).unwrap();

    let config = ZapConfig::new().unwrap();
    let mut zap = ZapWorker::new(config).unwrap();
    zap.load(&root).await.unwrap();

    assert_eq!(2, (*zap.toolchain_manager).read().unwrap().targets().len());
}
