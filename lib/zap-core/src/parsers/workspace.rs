use crate::*;
use anyhow::Context;
use log::*;
use std::path::PathBuf;

pub fn parse(
    toml: toml::Value,
    root: &PathBuf,
    toolchain_manager: &ToolchainManager,
) -> Result<Workspace, anyhow::Error> {
    let workspace = toml
        .get("workspace")
        .context("Workspace file must have a workspace section")?;
    let name = workspace
        .get("name")
        .context("Workspace must have a name field")?
        .as_str()
        .context("Workspace name field must be a string")?
        .to_string();

    let workspace = Workspace::new(name, root)?;

    let toolchain_archives = if let Some(toolchains) = toml.get("toolchains") {
        let table = toolchains.as_table().context(format!("Expected the [toolchains] section in your Workspace.toml to be a TOML table, but instead found a {}", toolchains.type_str()))?;
        parse_archives(table)?
    } else {
        vec![]
    };

    debug!("Found {} Toolchains:", toolchain_archives.len());
    for archive in toolchain_archives {
        debug!("* {:?}", archive);
        toolchain_manager.register_archive(archive);
    }

    Ok(workspace)
}

pub fn parse_archives(archives: &toml::value::Table) -> Result<Vec<Archive>, anyhow::Error> {
    let mut t = vec![];
    for (name, config) in archives {
        t.push(parse_archive(name.to_string(), config)?);
    }
    Ok(t)
}

pub fn parse_archive(name: String, cfg: &toml::Value) -> Result<Archive, anyhow::Error> {
    let archive = Archive::new().with_name(name);
    let archive = cfg
        .get("archive_url")
        .and_then(|x| x.as_str())
        .map(|url| archive.clone().with_url(url.to_string()).mark_as_source())
        .unwrap_or(archive);
    let archive = cfg
        .get("release_url")
        .and_then(|x| x.as_str())
        .map(|url| archive.clone().with_url(url.to_string()).mark_as_release())
        .unwrap_or(archive);
    let archive = cfg
        .get("release_name")
        .and_then(|x| x.as_str())
        .map(|name| {
            archive
                .clone()
                .with_name(name.to_string())
                .mark_as_release()
        })
        .unwrap_or(archive);
    let archive = cfg
        .get("prefix")
        .and_then(|x| x.as_str())
        .map(|prefix| archive.clone().with_prefix(prefix.to_string()))
        .unwrap_or(archive);
    let archive = cfg
        .get("sha1")
        .and_then(|x| x.as_str())
        .map(|sha1| archive.clone().with_sha1(sha1.to_string()))
        .unwrap_or(archive);
    Ok(archive)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demands_workspace_name() {
        let toml: toml::Value = r#"
[workspace]
[toolchains]
[dependencies]
        "#
        .parse::<toml::Value>()
        .unwrap();
        let workspace = parse(toml, &PathBuf::from("."), &ToolchainManager::default());
        assert_eq!(true, workspace.is_err());
    }

    #[test]
    fn parses_toml_into_workspace_struct() {
        let toml: toml::Value = r#"
[workspace]
name = "tiny_lib"

[toolchains]

[dependencies]
        "#
        .parse::<toml::Value>()
        .unwrap();
        let workspace = parse(toml, &PathBuf::from("."), &ToolchainManager::default()).unwrap();
        assert_eq!(workspace.name(), "tiny_lib");
    }

    #[test]
    fn allows_for_custom_toolchains() {
        let toml: toml::Value = r#"
    [workspace]
    name = "tiny_lib"

    [toolchains]
    erlang = { archive_url = "official", prefix = "otp-prefix" }
    gleam = { archive_url = "https://github.com/forked/gleam", sha1 = "sha1-test" }

    [dependencies]
            "#
        .parse::<toml::Value>()
        .unwrap();
        let toolchain_manager = ToolchainManager::default();
        let _workspace = parse(toml, &PathBuf::from("."), &toolchain_manager).unwrap();
        assert_eq!(
            "official",
            toolchain_manager.get_archive("erlang").unwrap().url()
        );
        assert_eq!(
            "otp-prefix",
            toolchain_manager.get_archive("erlang").unwrap().prefix()
        );
        assert_eq!(
            "https://github.com/forked/gleam",
            toolchain_manager.get_archive("gleam").unwrap().url()
        );
        assert_eq!(
            "sha1-test",
            toolchain_manager.get_archive("gleam").unwrap().sha1()
        );
    }
}
