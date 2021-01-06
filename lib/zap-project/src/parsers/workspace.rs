use anyhow::Context;
use std::path::PathBuf;
use zap_core::Workspace;

pub fn parse(toml: toml::Value, root: &PathBuf) -> Result<Workspace, anyhow::Error> {
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

    /*
    let toolchains = if let Some(toolchains) = toml.get("toolchains") {
        let table = toolchains.as_table().context(format!("Expected the [toolchains] section in your Workspace.toml to be a TOML table, but instead found a {}", toolchains.type_str()))?;
        parse_toolchains(*table)?
    } else {
        vec![]
    };

    debug!("Found {} Toolchains: {:?}", toolchains.len(), toolchains);
    for toolchain in toolchains {
        workspace.toolchain_manager_mut().register(toolchain);
    }
    */

    Ok(workspace)
}

/*
pub fn parse_toolchains(
    toolchains: toml::value::Table,
) -> Result<Vec<Box<dyn Toolchain>>, anyhow::Error> {
    let mut t = vec![];
    for (name, config) in toolchains {
        t.push(parse_toolchain(name, config)?);
    }
    Ok(t)
}

pub fn parse_toolchain(
    name: String,
    cfg: toml::Value,
) -> Result<Box<dyn Toolchain>, inyhow::Error> {
    let archive = Archive::new();
    let archive = cfg
        .get("archive_url")
        .and_then(|x| x.as_str())
        .map(|url| archive.clone().with_url(url.to_string()).as_source())
        .unwrap_or(archive);
    let archive = cfg
        .get("release_url")
        .and_then(|x| x.as_str())
        .map(|url| archive.clone().with_url(url.to_string()).as_release())
        .unwrap_or(archive);
    let archive = cfg
        .get("release_name")
        .and_then(|x| x.as_str())
        .map(|name| archive.clone().with_name(name.to_string()).as_release())
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
*/

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
        let workspace = parse(toml);
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
        let workspace = parse(toml).unwrap();
        assert_eq!(workspace.name(), "tiny_lib");
    }

    /*
        #[test]
        fn uses_defaults() {
            let toml: toml::Value = r#"
    [workspace]
    name = "tiny_lib"

    [toolchains]

    [dependencies]
            "#
            .parse::<toml::Value>()
            .unwrap();
            let workspace = Workspace::try_from(toml).unwrap();
            assert_eq!(
                workspace.toolchains().erlang().archive().url(),
                toolchains::erlang::Toolchain::default().archive().url()
            );
            assert_eq!(
                workspace.toolchains().erlang().archive().sha1(),
                toolchains::erlang::Toolchain::default().archive().sha1()
            );
            assert_eq!(
                workspace.toolchains().erlang().archive().prefix(),
                toolchains::erlang::Toolchain::default().archive().prefix()
            );
            assert_eq!(workspace.dependencies().len(), 0);
        }

        #[test]
        fn allows_for_custom_toolchains() {
            let toml: toml::Value = r#"
    [workspace]
    name = "tiny_lib"

    [toolchains]
    erlang = { archive_url = "master", prefix = "otp-master" }
    gleam = { archive_url = "https://github.com/forked/gleam", sha1 = "test" }

    [dependencies]
            "#
            .parse::<toml::Value>()
            .unwrap();
            let workspace = Workspace::try_from(toml).unwrap();
            assert_eq!(workspace.toolchains().erlang().archive().url(), "master");
            assert_eq!(
                workspace.toolchains().erlang().archive().prefix(),
                "otp-master"
            );
            assert_eq!(
                workspace.toolchains().gleam().archive().url(),
                "https://github.com/forked/gleam"
            );
            assert_eq!(workspace.toolchains().gleam().archive().sha1(), "test");
            assert_eq!(workspace.dependencies().len(), 0);
        }
        */
}
