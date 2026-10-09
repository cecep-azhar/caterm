use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SecretType {
    AwsAccessKey,
    GitHubToken,
    OpenAiKey,
    PrivateKey,
    GenericBearerToken,
    PasswordInUri,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedSecret {
    pub secret_type: SecretType,
    pub title: String,
    pub description: String,
    pub matched_snippet_masked: String,
    pub suggested_fix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasteCheckResult {
    pub contains_secret: bool,
    pub secrets: Vec<DetectedSecret>,
    pub sanitized_text: String,
    pub recommended_prefix: String,
}

struct SecretPattern {
    secret_type: SecretType,
    title: &'static str,
    description: &'static str,
    pattern: &'static str,
}

static PATTERNS: Lazy<Vec<(SecretPattern, Regex)>> = Lazy::new(|| {
    let patterns = vec![
        SecretPattern {
            secret_type: SecretType::AwsAccessKey,
            title: "AWS Access Key",
            description: "AWS 20-character Access Key ID (`AKIA...` or `ASIA...`) detected.",
            pattern: r#"\b(AKIA|ASIA|ABIA|ACCA)[0-9A-Z]{16}\b"#,
        },
        SecretPattern {
            secret_type: SecretType::GitHubToken,
            title: "GitHub Personal Access / OAuth Token",
            description: "GitHub Secret Token (`ghp_`, `gho_`, `ghu_`, `ghs_`, `ghr_`) detected.",
            pattern: r#"\bgh[pousr]_[A-Za-z0-9_]{36,255}\b"#,
        },
        SecretPattern {
            secret_type: SecretType::OpenAiKey,
            title: "OpenAI / AI Provider API Key",
            description: "OpenAI API secret (`sk-...` / `sk-proj-...`) detected.",
            pattern: r#"\bsk-[a-zA-Z0-9_-]{20,}\b"#,
        },
        SecretPattern {
            secret_type: SecretType::PrivateKey,
            title: "Private Cryptographic Key",
            description: "PEM-encoded private key (`BEGIN ... PRIVATE KEY`) detected.",
            pattern: r#"-----BEGIN [A-Z ]*PRIVATE KEY-----"#,
        },
        SecretPattern {
            secret_type: SecretType::PasswordInUri,
            title: "Password in Connection URI",
            description: "Plaintext credentials embedded in database or HTTP connection URI.",
            pattern: r#"(?i)\b(https?|postgres|postgresql|mysql|redis|mongodb)://[^:]+:([^@]+)@"#,
        },
    ];

    patterns
        .into_iter()
        .filter_map(|p| Regex::new(p.pattern).ok().map(|re| (p, re)))
        .collect()
});

fn mask_text(s: &str) -> String {
    if s.len() <= 8 {
        return "*".repeat(s.len());
    }
    let prefix = &s[..4];
    let suffix = &s[s.len() - 4..];
    format!("{}...{}", prefix, suffix)
}

/// Analyze text from clipboard before pasting into terminal or prompt
pub fn check_paste_content(text: &str) -> PasteCheckResult {
    let mut detected = Vec::new();

    for (rule, regex) in PATTERNS.iter() {
        for mat in regex.find_iter(text) {
            let matched_str = mat.as_str();
            let masked = mask_text(matched_str);
            detected.push(DetectedSecret {
                secret_type: rule.secret_type.clone(),
                title: rule.title.to_string(),
                description: rule.description.to_string(),
                matched_snippet_masked: masked,
                suggested_fix:
                    "Prefix command with a space (`HISTCONTROL=ignorespace`) or load from environment/vault"
                        .to_string(),
            });
        }
    }

    let contains_secret = !detected.is_empty();
    let sanitized_text = if contains_secret {
        // Prepend space if not already starts with space
        if text.starts_with(' ') {
            text.to_string()
        } else {
            format!(" {}", text)
        }
    } else {
        text.to_string()
    };

    PasteCheckResult {
        contains_secret,
        secrets: detected,
        sanitized_text,
        recommended_prefix: "HISTCONTROL=ignorespace".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_aws_key() {
        let res = check_paste_content("export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE");
        assert!(res.contains_secret);
        assert_eq!(res.secrets[0].secret_type, SecretType::AwsAccessKey);
        assert_eq!(
            res.sanitized_text,
            " export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE"
        );
    }

    #[test]
    fn test_detect_github_token() {
        let res = check_paste_content(
            "curl -H 'Authorization: token ghp_123456789012345678901234567890123456' https://api.github.com",
        );
        assert!(res.contains_secret);
        assert_eq!(res.secrets[0].secret_type, SecretType::GitHubToken);
    }

    #[test]
    fn test_detect_private_key() {
        let res = check_paste_content("-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0...");
        assert!(res.contains_secret);
        assert_eq!(res.secrets[0].secret_type, SecretType::PrivateKey);
    }

    #[test]
    fn test_safe_paste() {
        let res = check_paste_content("sudo systemctl restart nginx");
        assert!(!res.contains_secret);
        assert!(res.secrets.is_empty());
    }
}
