use std::ops::Deref;
use std::path::PathBuf;

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct Label {
    name: String,
}

impl Eq for Label {}

impl Deref for Label {
    type Target = String;
    fn deref(&self) -> &String {
        &self.name
    }
}

impl From<&str> for Label {
    fn from(name: &str) -> Label {
        Label::new(name.to_string())
    }
}

impl From<String> for Label {
    fn from(name: String) -> Label {
        Label::new(name.clone())
    }
}

impl ToString for Label {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl Default for Label {
    fn default() -> Label {
        "//...".into()
    }
}

impl Label {
    pub fn new(name: String) -> Label {
        let name = name.replace("\"", "");

        let is_wildcard = name.starts_with("//") && name.ends_with("...");
        let is_relative = name.starts_with(':');
        let is_abs_name = name.starts_with("//") && name.contains(':');

        let name = if is_relative || is_wildcard || is_abs_name {
            name
        } else {
            format!(":{}", name)
        };
        Label { name }
    }

    pub fn is_all(&self) -> bool {
        self.name.starts_with("//...")
    }

    pub fn is_absolute(&self) -> bool {
        self.name.starts_with("//")
    }

    pub fn canonicalize(self, path: &PathBuf) -> Label {
        if self.is_absolute() {
            self
        } else {
            let path = path
                .join(&self.name)
                .to_str()
                .unwrap()
                .replace("./", "")
                .replace("/:", ":");

            format!("//{}", path).into()
        }
    }
}
