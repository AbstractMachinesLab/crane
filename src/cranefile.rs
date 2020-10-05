use crate::build::{BuildRule, Rule};
use crate::rules::{
    BeamArchive, CaramelLibrary, ClojureLibrary, ElixirLibrary, ErlangLibrary, ErlangShell,
    GleamLibrary,
};
use anyhow::{Context, Error};
use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;
use std::vec::Vec;
use toml::Value;

pub const CRANEFILE: &str = "CRANE";

#[derive(Debug, Clone)]
pub struct Cranefile {
    path: PathBuf,
    contents: toml::Value,
    rules: Vec<BuildRule>,
}

impl Default for Cranefile {
    fn default() -> Self {
        Cranefile {
            path: PathBuf::from("./crane"),
            contents: "".parse::<Value>().unwrap(),
            rules: vec![],
        }
    }
}

impl Cranefile {
    pub fn from_file(path: PathBuf) -> Result<Cranefile, Error> {
        let contents = fs::read_to_string(&path)?.parse::<Value>()?;

        let parent = path.clone();
        let parent = &parent
            .parent()
            .context(format!("Could not get the parent of: {:?}", &path))?
            .to_path_buf();

        let erlang_shells: Result<Vec<BuildRule>, anyhow::Error> = contents
            .get("erlang_shell")
            .and_then(|m| m.as_array())
            .unwrap_or(&vec![])
            .iter()
            .cloned()
            .map(|x| ErlangShell::try_from((x, parent)).map(Rule::as_rule))
            .collect();
        let erlang_shells = erlang_shells?;

        let erlang_libraries: Result<Vec<BuildRule>, anyhow::Error> = contents
            .get("erlang_library")
            .and_then(|m| m.as_array())
            .unwrap_or(&vec![])
            .iter()
            .cloned()
            .map(|x| ErlangLibrary::try_from((x, parent)).map(Rule::as_rule))
            .collect();
        let erlang_libraries = erlang_libraries?;

        let clojure_libraries: Result<Vec<BuildRule>, anyhow::Error> = contents
            .get("clojure_library")
            .and_then(|m| m.as_array())
            .unwrap_or(&vec![])
            .iter()
            .cloned()
            .map(|x| ClojureLibrary::try_from((x, parent)).map(Rule::as_rule))
            .collect();
        let clojure_libraries = clojure_libraries?;

        let elixir_libraries: Result<Vec<BuildRule>, anyhow::Error> = contents
            .get("elixir_library")
            .and_then(|m| m.as_array())
            .unwrap_or(&vec![])
            .iter()
            .cloned()
            .map(|x| ElixirLibrary::try_from((x, parent)).map(Rule::as_rule))
            .collect();
        let elixir_libraries = elixir_libraries?;

        let gleam_libraries: Result<Vec<BuildRule>, anyhow::Error> = contents
            .get("gleam_library")
            .and_then(|m| m.as_array())
            .unwrap_or(&vec![])
            .iter()
            .cloned()
            .map(|x| GleamLibrary::try_from((x, parent)).map(Rule::as_rule))
            .collect();
        let gleam_libraries = gleam_libraries?;

        let caramel_libraries: Result<Vec<BuildRule>, anyhow::Error> = contents
            .get("caramel_library")
            .and_then(|m| m.as_array())
            .unwrap_or(&vec![])
            .iter()
            .cloned()
            .map(|x| CaramelLibrary::try_from((x, parent)).map(Rule::as_rule))
            .collect();
        let caramel_libraries = caramel_libraries?;

        let beam_archives: Result<Vec<BuildRule>, anyhow::Error> = contents
            .get("beam_archive")
            .and_then(|m| m.as_array())
            .unwrap_or(&vec![])
            .iter()
            .cloned()
            .map(|x| BeamArchive::try_from((x, parent)).map(Rule::as_rule))
            .collect();
        let beam_archives = beam_archives?;

        let rules: Vec<BuildRule> = vec![
            beam_archives,
            caramel_libraries,
            clojure_libraries,
            elixir_libraries,
            erlang_libraries,
            erlang_shells,
            gleam_libraries,
        ]
        .iter()
        .flatten()
        .cloned()
        .collect();

        Ok(Cranefile {
            path,
            contents,
            rules,
        })
    }

    pub fn rules(&self) -> Vec<BuildRule> {
        self.rules.clone()
    }
}
