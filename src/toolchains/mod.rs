use log::debug;
use std::convert::TryFrom;
use std::path::PathBuf;

mod archive;
pub mod caramel;
pub mod clojerl;
pub mod elixir;
pub mod erlang;
pub mod gleam;

#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub enum ToolchainName {
    Clojure,
    Elixir,
    Erlang,
    Gleam,
    Caramel,
}

impl Eq for ToolchainName {}

pub struct ToolchainBuilder {
    name: String,
    archive: archive::Archive,
    is_cached: Box<dyn Fn() -> Result<bool, anyhow::Error>>,
    build_toolchain: Box<dyn Fn() -> Result<(), anyhow::Error>>,
}

impl ToolchainBuilder {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn archive(&self) -> &archive::Archive {
        &self.archive
    }

    pub fn is_cached(&self) -> Result<bool, anyhow::Error> {
        (self.is_cached)()
    }

    pub fn build_toolchain(&self) -> Result<(), anyhow::Error> {
        (self.build_toolchain)()
    }
}

pub trait IntoToolchainBuilder {
    fn toolchain_builder(&self) -> ToolchainBuilder;
}

#[derive(Debug, Clone, Default)]
pub struct Toolchain {
    clojerl: clojerl::Toolchain,
    elixir: elixir::Toolchain,
    erlang: erlang::Toolchain,
    gleam: gleam::Toolchain,
    caramel: caramel::Toolchain,
}

impl Toolchain {
    pub fn clojerl(&self) -> clojerl::Toolchain {
        self.clojerl.clone()
    }

    pub fn elixir(&self) -> elixir::Toolchain {
        self.elixir.clone()
    }

    pub fn erlang(&self) -> erlang::Toolchain {
        self.erlang.clone()
    }

    pub fn gleam(&self) -> gleam::Toolchain {
        self.gleam.clone()
    }

    pub fn caramel(&self) -> caramel::Toolchain {
        self.caramel.clone()
    }

    pub fn set_root(self, root: PathBuf) -> Toolchain {
        Toolchain {
            clojerl: self.clojerl.with_root(root.clone()),
            elixir: self.elixir.with_root(root.clone()),
            erlang: self.erlang.with_root(root.clone()),
            caramel: self.caramel.with_root(root.clone()),
            gleam: self.gleam.with_root(root),
        }
    }

    pub fn ready_toolchain(&self, toolchain: ToolchainName) -> Result<(), anyhow::Error> {
        let root = match toolchain {
            ToolchainName::Erlang => self.erlang().root().clone(),
            ToolchainName::Clojure => self.clojerl().root().clone(),
            ToolchainName::Elixir => self.elixir().root().clone(),
            ToolchainName::Gleam => self.gleam().root().clone(),
            ToolchainName::Caramel => self.caramel().root().clone(),
        };

        let no_prefix = match toolchain {
            ToolchainName::Erlang => self.erlang().archive().prefix(),
            ToolchainName::Clojure => self.clojerl().archive().prefix(),
            ToolchainName::Elixir => self.elixir().archive().prefix(),
            ToolchainName::Gleam => self.gleam().archive().prefix(),
            ToolchainName::Caramel => self.caramel().archive().prefix(),
        }
        .is_empty();

        let builder = match toolchain {
            ToolchainName::Erlang => self.erlang().toolchain_builder(),
            ToolchainName::Clojure => self.clojerl().toolchain_builder(),
            ToolchainName::Elixir => self.elixir().toolchain_builder(),
            ToolchainName::Gleam => self.gleam().toolchain_builder(),
            ToolchainName::Caramel => self.caramel().toolchain_builder(),
        };

        let download_root = if no_prefix {
            root.to_path_buf()
        } else {
            root.parent().unwrap().to_path_buf()
        };
        let archive = builder.archive().clone();
        &archive.validate();
        if !archive.is_cached(&download_root)? {
            archive.download(&download_root)?;
            archive.checksum(&download_root)?;
            archive.unpack(&download_root)?;
        }

        match builder.is_cached() {
            Ok(true) => Ok(()),
            _ => builder.build_toolchain(),
        }
    }
}

impl TryFrom<toml::Value> for Toolchain {
    type Error = anyhow::Error;

    fn try_from(toml: toml::Value) -> Result<Toolchain, anyhow::Error> {
        let erlang = {
            let t = erlang::Toolchain::default();
            let a = override_archive_from_toml(t.archive().clone(), toml.get("erlang"));
            t.with_archive(a)
        };

        let elixir = {
            let t = elixir::Toolchain::default();
            let a = override_archive_from_toml(t.archive().clone(), toml.get("elixir"));
            t.with_archive(a)
        };

        let gleam = {
            let t = gleam::Toolchain::default();
            let a = override_archive_from_toml(t.archive().clone(), toml.get("gleam"));
            t.with_archive(a)
        };

        let clojerl = {
            let t = clojerl::Toolchain::default();
            let a = override_archive_from_toml(t.archive().clone(), toml.get("clojerl"));
            t.with_archive(a)
        };

        let caramel = {
            let t = caramel::Toolchain::default();
            let a = override_archive_from_toml(t.archive().clone(), toml.get("caramel"));
            t.with_archive(a)
        };

        Ok(Toolchain {
            caramel,
            clojerl,
            gleam,
            elixir,
            erlang,
        })
    }
}

fn override_archive_from_toml(
    archive: archive::Archive,
    toml: Option<&toml::Value>,
) -> archive::Archive {
    match toml {
        Some(v) => {
            let archive = v
                .get("archive_url")
                .and_then(|x| x.as_str())
                .map(|url| archive.clone().with_url(url.to_string()).as_source())
                .unwrap_or(archive);
            let archive = v
                .get("release_url")
                .and_then(|x| x.as_str())
                .map(|url| archive.clone().with_url(url.to_string()).as_release())
                .unwrap_or(archive);
            let archive = v
                .get("release_name")
                .and_then(|x| x.as_str())
                .map(|name| archive.clone().with_name(name.to_string()).as_release())
                .unwrap_or(archive);
            let archive = v
                .get("prefix")
                .and_then(|x| x.as_str())
                .map(|prefix| archive.clone().with_prefix(prefix.to_string()))
                .unwrap_or(archive);
            let archive = v
                .get("sha1")
                .and_then(|x| x.as_str())
                .map(|sha1| archive.clone().with_sha1(sha1.to_string()))
                .unwrap_or(archive);
            archive
        }
        None => archive,
    }
}
