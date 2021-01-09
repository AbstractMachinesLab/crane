use super::Label;
use anyhow::*;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum CfgValueType {
    String,
    Label,
    File,
    List(Box<CfgValueType>),
}

#[derive(Debug, Clone)]
pub enum CfgValue {
    String(String),
    Label(Label),
    File(PathBuf),
    List(Vec<CfgValue>),
}

impl From<(serde_json::Value, CfgValueType)> for CfgValue {
    fn from(spec: (serde_json::Value, CfgValueType)) -> CfgValue {
        let (json, type_) = spec;
        match json {
            serde_json::Value::String(string) => match type_ {
                CfgValueType::String => CfgValue::String(string),
                CfgValueType::Label => CfgValue::Label(Label::new(&string)),
                CfgValueType::File => CfgValue::File(PathBuf::from(string)),
                CfgValueType::List(_) => panic!("Oops!"),
            },
            serde_json::Value::Array(parts) => match type_ {
                CfgValueType::List(element_type) => CfgValue::List(
                    parts
                        .iter()
                        .map(|e| (e.clone(), *element_type.clone()).into())
                        .collect(),
                ),
                _ => panic!("oops!"),
            },
            _ => panic!("oops!"),
        }
    }
}

impl Into<serde_json::Value> for CfgValue {
    fn into(self) -> serde_json::Value {
        match self {
            CfgValue::String(string) => serde_json::Value::String(string),
            CfgValue::Label(label) => serde_json::Value::String(label.to_string()),
            CfgValue::File(path) => serde_json::Value::String(path.to_str().unwrap().to_string()),
            CfgValue::List(parts) => {
                serde_json::Value::Array(parts.iter().map(|e| e.clone().into()).collect())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigSpec(pub HashMap<String, CfgValueType>);

impl ConfigSpec {
    pub fn get(&self, key: &str) -> Option<&CfgValueType> {
        self.0.get(key)
    }

    pub fn keys(&self) -> Vec<String> {
        self.0.keys().into_iter().map(|k| k.to_string()).collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RuleConfig(pub DashMap<String, CfgValue>);

impl RuleConfig {
    pub fn as_map(&self) -> &DashMap<String, CfgValue> {
        &self.0
    }

    pub fn insert(&self, key: String, val: CfgValue) -> &RuleConfig {
        self.0.insert(key, val);
        self
    }

    pub fn insert_str(&self, key: String, val: &str) -> &RuleConfig {
        self.insert(key, CfgValue::String(val.to_string()))
    }

    pub fn insert_path(&self, key: String, val: &PathBuf) -> &RuleConfig {
        self.insert(key, CfgValue::File(val.clone()))
    }

    pub fn get_label_list(&self, key: &str) -> Result<Vec<Label>, anyhow::Error> {
        let entry = self
            .0
            .get(key)
            .context(format!("Could not find key {:?}", key))?;

        if let CfgValue::List(elements) = entry.value() {
            let mut labels = vec![];
            for el in elements {
                if let CfgValue::Label(l) = el {
                    labels.push(l.clone());
                } else {
                    return Err(anyhow!(
                        "Expected all values in {:?} to be labels, but found {:?}",
                        key,
                        el
                    ));
                }
            }
            Ok(labels)
        } else {
            Err(anyhow!(
                "Expected key {:?} to be a label list but found {:?} instead.",
                key,
                entry.value()
            ))
        }
    }

    pub fn get_file_list(&self, key: &str) -> Result<Vec<PathBuf>, anyhow::Error> {
        let entry = self
            .0
            .get(key)
            .context(format!("Could not find key {:?}", key))?;

        if let CfgValue::List(elements) = entry.value() {
            let mut paths = vec![];
            for el in elements {
                if let CfgValue::File(path) = el {
                    paths.push(path.clone());
                } else {
                    return Err(anyhow!(
                        "Expected all values in {:?} to be file paths, but found {:?}",
                        key,
                        el
                    ));
                }
            }
            Ok(paths)
        } else {
            Err(anyhow!(
                "Expected key {:?} to be a file list but found {:?} instead.",
                key,
                entry.value()
            ))
        }
    }
}

impl Into<serde_json::Value> for RuleConfig {
    fn into(self) -> serde_json::Value {
        let mut map: serde_json::map::Map<String, serde_json::Value> = serde_json::map::Map::new();

        for entry in self.0.iter() {
            map.insert(entry.key().to_string(), entry.value().clone().into());
        }

        serde_json::Value::Object(map)
    }
}
