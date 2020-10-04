use std::path::PathBuf;

static WILDCARD: &str = "//...";

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum Label {
    Wildcard,
    Relative { name: String },
    Absolute { name: String, path: PathBuf },
}

impl Eq for Label {}

impl From<&str> for Label {
    fn from(name: &str) -> Label {
        Label::new(name.to_string())
    }
}

impl From<String> for Label {
    fn from(name: String) -> Label {
        Label::new(name)
    }
}

impl ToString for Label {
    fn to_string(&self) -> String {
        match self {
            Label::Wildcard => WILDCARD.to_string(),
            Label::Relative { name } => format!(":{}", name),
            Label::Absolute { name, path } => format!("//{}:{}", path.to_str().unwrap(), name),
        }
    }
}

impl Default for Label {
    fn default() -> Label {
        WILDCARD.into()
    }
}

impl Label {
    pub fn new(name: String) -> Label {
        let name = name.replace("\"", "");

        let is_wildcard = name.starts_with("//") && name.ends_with("...");
        let is_abs_name = name.starts_with("//") && name.contains(':');

        if is_wildcard {
            return Label::Wildcard;
        }

        if is_abs_name {
            let parts: Vec<&str> = name.split(":").collect();
            return Label::Absolute {
                path: PathBuf::from(parts[0].strip_prefix("//").unwrap()),
                name: parts[1].to_string(),
            };
        }

        return Label::Relative {
            name: if name.starts_with(":") {
                name.strip_prefix(":").unwrap().to_string()
            } else {
                name
            },
        };
    }

    pub fn name(&self) -> String {
        match self {
            Label::Wildcard => WILDCARD.to_string(),
            Label::Relative { name } => name.clone(),
            Label::Absolute { name, .. } => name.clone(),
        }
    }

    pub fn path(&self) -> PathBuf {
        match self {
            Label::Wildcard => PathBuf::from("."),
            Label::Relative { .. } => PathBuf::from("."),
            Label::Absolute { path, .. } => path.to_path_buf(),
        }
    }

    pub fn is_all(&self) -> bool {
        match self {
            Label::Wildcard => true,
            _ => false,
        }
    }

    pub fn is_absolute(&self) -> bool {
        match self {
            Label::Absolute { .. } => true,
            _ => false,
        }
    }

    pub fn canonicalize(self, path: &PathBuf) -> Label {
        match self {
            Label::Relative { name } => Label::Absolute {
                name,
                path: if path.starts_with("./") {
                    path.strip_prefix("./")
                        .expect(&format!(
                            "Could not remove relative prefix from relative path: {:?}",
                            &path
                        ))
                        .clone()
                        .to_path_buf()
                } else {
                    path.clone()
                },
            },
            _ => self,
        }
    }
}
