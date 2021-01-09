use super::*;

#[derive(Clone, Debug)]
pub struct Toolchain {
    target: Target,
    rule: Rule,
    archive: Archive,
}

impl Toolchain {
    pub fn new(rule: Rule, archive: Archive) -> Toolchain {
        let cfg = Toolchain::archive_to_config(&archive);
        let target = Target::new(Label::new(rule.name()), &rule, cfg);
        Toolchain {
            rule,
            archive,
            target,
        }
    }

    pub fn label(&self) -> &Label {
        &self.target.label()
    }

    pub fn as_target(&self) -> &Target {
        &self.target
    }

    pub fn archive_to_config(archive: &Archive) -> RuleConfig {
        let cfg = RuleConfig::default();

        cfg.insert_str("name".to_string(), archive.name());
        cfg.insert_str("prefix".to_string(), archive.prefix());
        cfg.insert_str("sha1".to_string(), archive.sha1());
        cfg.insert_str("tag".to_string(), archive.tag());
        cfg.insert_path("unarchivedRoot".to_string(), archive.unarchived_root());
        cfg.insert_str("url".to_string(), &archive.url());

        cfg
    }
}
