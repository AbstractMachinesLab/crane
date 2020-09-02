use crate::build::{BuildRule, Rule};
use crate::rules::{ClojureLibrary, ElixirLibrary, ErlangLibrary, ErlangShell, GleamLibrary};
use anyhow::Error;
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

        let parent = &path.parent().unwrap().to_path_buf();

        let erlang_shells: Result<Vec<BuildRule>, anyhow::Error> = contents
            .get("erlang_shell")
            .unwrap_or(&Value::Array(vec![]))
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .cloned()
            .map(|x| ErlangShell::try_from((x, parent)).map(Rule::as_rule))
            .collect();
        let erlang_shells = erlang_shells?;

        let erlang_libraries: Result<Vec<BuildRule>, anyhow::Error> = contents
            .get("erlang_library")
            .unwrap_or(&Value::Array(vec![]))
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .cloned()
            .map(|x| ErlangLibrary::try_from((x, parent)).map(Rule::as_rule))
            .collect();
        let erlang_libraries = erlang_libraries?;

        let clojure_libraries: Result<Vec<BuildRule>, anyhow::Error> = contents
            .get("clojure_library")
            .unwrap_or(&Value::Array(vec![]))
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .cloned()
            .map(|x| ClojureLibrary::try_from((x, parent)).map(Rule::as_rule))
            .collect();
        let clojure_libraries = clojure_libraries?;

        let elixir_libraries: Result<Vec<BuildRule>, anyhow::Error> = contents
            .get("elixir_library")
            .unwrap_or(&Value::Array(vec![]))
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .cloned()
            .map(|x| ElixirLibrary::try_from((x, parent)).map(Rule::as_rule))
            .collect();
        let elixir_libraries = elixir_libraries?;

        let gleam_libraries: Result<Vec<BuildRule>, anyhow::Error> = contents
            .get("gleam_library")
            .unwrap_or(&Value::Array(vec![]))
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .cloned()
            .map(|x| GleamLibrary::try_from((x, parent)).map(Rule::as_rule))
            .collect();
        let gleam_libraries = gleam_libraries?;
        /*
        &contents
        .get("gleam_library")
        .unwrap_or(&Value::Array(vec![]))
        .into::<GleamLibrary>()
        .as_rule(),
        &contents
        .get("hamler_library")
        .unwrap_or(&Value::Array(vec![]))
        .into::<HamlerLibrary>()
        .as_rule(),
        &contents
        .get("luerl_library")
        .unwrap_or(&Value::Array(vec![]))
        .into::<LuaLibrary>()
        .as_rule(),
        */

        let rules: Vec<BuildRule> = vec![
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
