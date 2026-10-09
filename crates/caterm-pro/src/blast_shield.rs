use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DangerLevel {
    Warning,
    Critical,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlastShieldRisk {
    pub matched_rule: String,
    pub level: DangerLevel,
    pub reason: String,
    pub suggested_alternative: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlastShieldCheckResult {
    pub is_destructive: bool,
    pub command: String,
    pub risk: Option<BlastShieldRisk>,
}

struct Rule {
    name: &'static str,
    pattern: &'static str,
    level: DangerLevel,
    reason: &'static str,
    suggested_alternative: Option<&'static str>,
}

static RULES: Lazy<Vec<(Rule, Regex)>> = Lazy::new(|| {
    let rules = vec![
        Rule {
            name: "RM_ROOT_OR_RECURSIVE_FORCE",
            pattern: r#"(?i)\brm\s+-[a-zA-Z0-9]*r[a-zA-Z0-9]*f[a-zA-Z0-9]*\s+(/\s*$|/\*|\.\s*$|\.\*|~|/\w+)"#,
            level: DangerLevel::Blocked,
            reason: "Attempting recursive force deletion (`rm -rf`) on root or critical system directories.",
            suggested_alternative: Some("Use trash-cli or double check target absolute directory"),
        },
        Rule {
            name: "RM_FORCE_GENERIC",
            pattern: r#"(?i)\brm\s+(-[a-zA-Z0-9]*r[a-zA-Z0-9]*f|-rf|--recursive\s+--force)\b"#,
            level: DangerLevel::Critical,
            reason: "Command contains recursive and forced deletion (`rm -rf`).",
            suggested_alternative: Some("Specify explicit targets and remove `-f` flag to verify"),
        },
        Rule {
            name: "DROP_DATABASE_OR_TABLE",
            pattern: r#"(?i)\b(DROP\s+DATABASE|DROP\s+SCHEMA|DROP\s+TABLE|TRUNCATE\s+TABLE)\b"#,
            level: DangerLevel::Blocked,
            reason: "Direct destructive SQL command (DROP / TRUNCATE) detected.",
            suggested_alternative: Some("Take a SQL backup snapshot before executing schema drops"),
        },
        Rule {
            name: "MKFS_FORMAT_DISK",
            pattern: r#"(?i)\bmkfs(\.[a-zA-Z0-9_-]+)?\s+/dev/"#,
            level: DangerLevel::Blocked,
            reason: "Formatting partition or raw block device (`mkfs`).",
            suggested_alternative: Some("Confirm target disk device path with `lsblk`"),
        },
        Rule {
            name: "DD_RAW_WRITE",
            pattern: r#"(?i)\bdd\s+.*(of=/dev/(sd[a-z]|nvme[0-9]n[0-9]|vd[a-z]|hd[a-z]))"#,
            level: DangerLevel::Critical,
            reason: "Direct block device overwrite using `dd of=/dev/...` will destroy existing file tables.",
            suggested_alternative: Some("Verify output target device (`of=`) before proceeding"),
        },
        Rule {
            name: "FLUSH_IPTABLES_FIREWALL",
            pattern: r#"(?i)\biptables\s+(-[A-Z0-9]*F|-F|--flush)\b"#,
            level: DangerLevel::Warning,
            reason: "Flushing iptables firewall rules might lock you out of remote SSH.",
            suggested_alternative: Some(
                "Ensure default ACCEPT policy or console access before flushing",
            ),
        },
        Rule {
            name: "CHMOD_777_RECURSIVE",
            pattern: r#"(?i)\bchmod\s+(-[a-zA-Z0-9]*R|--recursive)\s+(0?777|777)\b"#,
            level: DangerLevel::Warning,
            reason: "Recursive 777 permissions exposes files to arbitrary execution and write access by all users.",
            suggested_alternative: Some(
                "Use `chmod -R 755` for directories or `chmod -R 644` for files",
            ),
        },
        Rule {
            name: "SHRED_ZERO_DISK",
            pattern: r#"(?i)\b(shred|wipefs)\s+.*(/dev/)"#,
            level: DangerLevel::Blocked,
            reason: "Direct block disk wiping (`shred`/`wipefs`).",
            suggested_alternative: Some("Verify disk device node with `fdisk -l`"),
        },
    ];

    rules
        .into_iter()
        .filter_map(|r| Regex::new(r.pattern).ok().map(|re| (r, re)))
        .collect()
});

/// Analyze a terminal input command before dispatching to PTY
pub fn check_command_safety(command: &str) -> BlastShieldCheckResult {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return BlastShieldCheckResult {
            is_destructive: false,
            command: command.to_string(),
            risk: None,
        };
    }

    for (rule, regex) in RULES.iter() {
        if regex.is_match(trimmed) {
            return BlastShieldCheckResult {
                is_destructive: true,
                command: command.to_string(),
                risk: Some(BlastShieldRisk {
                    matched_rule: rule.name.to_string(),
                    level: rule.level.clone(),
                    reason: rule.reason.to_string(),
                    suggested_alternative: rule.suggested_alternative.map(|s| s.to_string()),
                }),
            };
        }
    }

    BlastShieldCheckResult {
        is_destructive: false,
        command: command.to_string(),
        risk: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rm_rf_root() {
        let res = check_command_safety("rm -rf /");
        assert!(res.is_destructive);
        assert_eq!(res.risk.unwrap().level, DangerLevel::Blocked);
    }

    #[test]
    fn test_rm_rf_dir() {
        let res = check_command_safety("rm -rf ./node_modules");
        assert!(res.is_destructive);
        assert_eq!(res.risk.unwrap().level, DangerLevel::Critical);
    }

    #[test]
    fn test_drop_database() {
        let res = check_command_safety("DROP DATABASE production;");
        assert!(res.is_destructive);
        assert_eq!(res.risk.unwrap().level, DangerLevel::Blocked);
    }

    #[test]
    fn test_safe_command() {
        let res = check_command_safety("ls -la /var/log");
        assert!(!res.is_destructive);
    }
}
