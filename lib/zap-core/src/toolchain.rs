use super::*;

#[derive(Clone, Debug)]
pub struct Toolchain {
    target: Target,
    rule: Rule,
}

impl Toolchain {
    pub fn new(rule: Rule, archive: Archive) -> Toolchain {
        let cfg = Toolchain::archive_to_config(&archive);
        let target = Target::global(Label::new(rule.name()), &rule, cfg, archive);
        Toolchain { rule, target }
    }

    pub fn label(&self) -> &Label {
        &self.target.label()
    }

    pub fn as_target(&self) -> &Target {
        &self.target
    }

    pub fn archive_to_config(archive: &Archive) -> RuleConfig {
        let cfg = RuleConfig::default();

        cfg.insert_path("unarchivedRoot".to_string(), &archive.unarchived_root());
        cfg.insert_str("archiveKind".to_string(), &archive.kind());
        cfg.insert_str("archiveName".to_string(), archive.name());
        cfg.insert_str("archivePrefix".to_string(), archive.prefix());
        cfg.insert_str("archiveSha1".to_string(), archive.sha1());
        cfg.insert_str("archiveUrl".to_string(), &archive.url());

        cfg
    }
}
