use std::path::PathBuf;

static COLON: char = ':';
static DOT: &str = ".";

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
        Label::new(name)
    }
}

impl From<String> for Label {
    fn from(name: String) -> Label {
        Label::new(&name)
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
    pub fn new(name: &str) -> Label {
        let name = name.replace("\"", "");

        let is_wildcard = name.starts_with("//") && name.ends_with("...");
        let is_abs_name = name.starts_with("//") && name.contains(COLON);

        if is_wildcard {
            return Label::Wildcard;
        }

        if is_abs_name {
            let parts: Vec<&str> = name.split(COLON).collect();
            return Label::Absolute {
                path: PathBuf::from(parts[0].strip_prefix("//").unwrap()),
                name: parts[1].to_string(),
            };
        }

        Label::Relative {
            name: if name.starts_with(COLON) {
                name.strip_prefix(COLON).unwrap().to_string()
            } else {
                name
            },
        }
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
            Label::Wildcard => PathBuf::from(DOT),
            Label::Relative { .. } => PathBuf::from(DOT),
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
            Label::Absolute { .. } | Label::Wildcard => true,
            _ => false,
        }
    }

    pub fn canonicalize(self, path: &PathBuf) -> Label {
        match self {
            Label::Relative { name } => Label::Absolute {
                name,
                path: if path.starts_with("./") {
                    path.strip_prefix("./")
                        .unwrap_or_else(|_| {
                            panic!(
                                "Could not remove relative prefix from relative path: {:?}",
                                &path
                            )
                        })
                        .to_path_buf()
                } else {
                    path.clone()
                },
            },
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_relative_paths() {
        let path = ":hello";
        let l1 = Label::new(&path);
        let l2: Label = path.into();
        let l3: Label = path.to_string().into();
        assert_eq!(path, l1.to_string());
        assert_eq!(false, l1.is_absolute());
        assert_eq!(false, l1.is_all());
        assert_eq!(path, l2.to_string());
        assert_eq!(false, l2.is_absolute());
        assert_eq!(false, l2.is_all());
        assert_eq!(path, l3.to_string());
        assert_eq!(false, l3.is_absolute());
        assert_eq!(false, l3.is_all());
    }

    #[test]
    fn parses_absolute_paths() {
        let path = "//my/path:hello";
        let l1 = Label::new(&path);
        let l2: Label = path.into();
        let l3: Label = path.to_string().into();
        assert_eq!(path, l1.to_string());
        assert_eq!(true, l1.is_absolute());
        assert_eq!(false, l1.is_all());
        assert_eq!(path, l2.to_string());
        assert_eq!(true, l2.is_absolute());
        assert_eq!(false, l2.is_all());
        assert_eq!(path, l3.to_string());
        assert_eq!(true, l3.is_absolute());
        assert_eq!(false, l3.is_all());
    }

    #[test]
    fn parses_wildcard_path() {
        let path = "//...";
        let l1 = Label::new(&path);
        let l2: Label = path.into();
        let l3: Label = path.to_string().into();
        assert_eq!(path, l1.to_string());
        assert_eq!(true, l1.is_absolute());
        assert_eq!(true, l1.is_all());
        assert_eq!(path, l2.to_string());
        assert_eq!(true, l2.is_absolute());
        assert_eq!(true, l2.is_all());
        assert_eq!(path, l3.to_string());
        assert_eq!(true, l3.is_absolute());
        assert_eq!(true, l3.is_all());
    }
}
