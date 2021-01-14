use assert_cmd::Command;
use std::path::PathBuf;

fn run(parts: &[&str]) -> String {
    let root = std::fs::canonicalize(PathBuf::from(".")).unwrap();
    let parts = parts.to_vec();
    let mut cmd = Command::cargo_bin(parts[0]).unwrap();
    cmd.env("RUST_BACKTRACE", "1");
    cmd.env("ZAP_LOG", "info");
    cmd.env("ZAP_HOME", "../_zap_home");
    cmd.current_dir(std::fs::canonicalize(PathBuf::from("./tests/sample_project")).unwrap());
    cmd.arg("--quiet");
    for arg in &parts[1..] {
        cmd.arg(arg);
    }
    let bytes = cmd.assert().get_output().stdout.to_vec();

    std::env::set_current_dir(root).unwrap();

    String::from_utf8(bytes).unwrap()
}

#[test]
#[ignore]
pub fn zap_workspace_info() {
    insta::assert_snapshot!(run(&["zap", "workspace", "info"]))
}

#[test]
#[ignore]
pub fn zap_toolchains_list_active() {
    insta::assert_snapshot!(run(&["zap", "toolchains", "list-active"]))
}

#[test]
#[ignore]
pub fn zap_toolchains_list_archives() {
    insta::assert_snapshot!(run(&["zap", "toolchains", "list-archives"]))
}

#[test]
#[ignore]
pub fn zap_toolchains_list_available() {
    insta::assert_snapshot!(run(&["zap", "toolchains", "list-available"]))
}

#[test]
#[ignore]
pub fn zap_targets_list() {
    insta::assert_snapshot!(run(&["zap", "targets", "list"]))
}

#[test]
#[ignore]
pub fn zap_rules_list() {
    insta::assert_snapshot!(run(&["zap", "rules", "list"]))
}

#[test]
#[ignore]
pub fn zap_rules_dump_actions() {
    insta::assert_snapshot!(run(&["zap", "rules", "dump-actions", "//..."]))
}

#[test]
#[ignore]
pub fn zap_rules_dump_outputs() {
    insta::assert_snapshot!(run(&["zap", "rules", "dump-outputs", "//..."]))
}

#[test]
#[ignore]
pub fn zap_dep_graph_print() {
    // insta::assert_snapshot!(run(&["zap", "dep-graph", "print", "//..."]))
}

#[test]
#[ignore]
pub fn zap_cache_clear() {
    // insta::assert_snapshot!(run(&["zap", "cache", "clear", "//a:lib"]))
}

#[test]
#[ignore]
pub fn zap_build() {
    insta::assert_snapshot!(run(&["zap"]));
    insta::assert_snapshot!(run(&["zap", "build"]));
    insta::assert_snapshot!(run(&["zap", "build", "//..."]));
    insta::assert_snapshot!(run(&["zap", "build", "//a:lib"]));
}
