use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AuditSeverity {
    Info,
    Warning,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaptopPostureFinding {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: AuditSeverity,
    pub recommendation: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaptopPostureReport {
    pub timestamp: String,
    pub overall_score: u32,
    pub findings: Vec<LaptopPostureFinding>,
    pub scanned_ssh_keys: usize,
    pub scanned_env_files: usize,
    pub scanned_listening_ports: usize,
}

/// Audit local ~/.ssh permissions
fn check_ssh_directory() -> (Vec<LaptopPostureFinding>, usize) {
    let mut findings = Vec::new();
    let mut scanned_keys = 0;

    let home_dir = match dirs::home_dir() {
        Some(d) => d,
        None => return (findings, 0),
    };

    let ssh_dir = home_dir.join(".ssh");
    if !ssh_dir.exists() {
        return (findings, 0);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(meta) = fs::metadata(&ssh_dir) {
            let mode = meta.permissions().mode() & 0o777;
            if mode != 0o700 {
                findings.push(LaptopPostureFinding {
                    id: "SSH_DIR_PERMS".to_string(),
                    title: "Insecure ~/.ssh Directory Permissions".to_string(),
                    description: format!(
                        "~/.ssh has permission {:o} (should be 700). Other local users may read SSH configs.",
                        mode
                    ),
                    severity: AuditSeverity::High,
                    recommendation: "Run `chmod 700 ~/.ssh` immediately.".to_string(),
                    details: Some(format!("Current mode: {:o}", mode)),
                });
            }
        }

        if let Ok(entries) = fs::read_dir(&ssh_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let filename = path.file_name().unwrap_or_default().to_string_lossy();
                        if filename.starts_with("id_") && !filename.ends_with(".pub") {
                            scanned_keys += 1;
                            if let Ok(meta) = fs::metadata(&path) {
                                let mode = meta.permissions().mode() & 0o777;
                                if mode != 0o600 && mode != 0o400 {
                                    findings.push(LaptopPostureFinding {
                                        id: format!("SSH_KEY_PERMS_{}", filename),
                                        title: format!(
                                            "Overly Permissive Private Key ({})",
                                            filename
                                        ),
                                        description: format!(
                                            "Private key file `{}` has mode {:o} (recommended: 600).",
                                            filename, mode
                                        ),
                                        severity: AuditSeverity::Critical,
                                        recommendation: format!("Run `chmod 600 ~/.ssh/{}`", filename),
                                        details: Some(format!("Path: {}", path.display())),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (findings, scanned_keys)
}

/// Recursively find and check `.env` files for unencrypted secrets in current working directory / common project paths
fn check_env_files() -> (Vec<LaptopPostureFinding>, usize) {
    let mut findings = Vec::new();
    let mut scanned_env = 0;

    let target_paths = vec![
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")),
    ];

    for root in target_paths {
        let env_candidates = vec![
            root.join(".env"),
            root.join(".env.local"),
            root.join(".env.production"),
        ];

        for env_path in env_candidates {
            if env_path.exists() && env_path.is_file() {
                scanned_env += 1;
                if let Ok(content) = fs::read_to_string(&env_path) {
                    let has_secret_pattern = content.contains("KEY=")
                        || content.contains("SECRET=")
                        || content.contains("PASSWORD=")
                        || content.contains("TOKEN=");

                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(meta) = fs::metadata(&env_path) {
                            let mode = meta.permissions().mode() & 0o777;
                            if (mode & 0o044) != 0 && has_secret_pattern {
                                findings.push(LaptopPostureFinding {
                                    id: format!("ENV_FILE_WORLD_READABLE_{}", env_path.display()),
                                    title: "Plaintext Secret in World-Readable .env File".to_string(),
                                    description: format!(
                                        "File `{}` containing credentials has mode {:o} (world/group readable).",
                                        env_path.display(),
                                        mode
                                    ),
                                    severity: AuditSeverity::High,
                                    recommendation: format!(
                                        "Run `chmod 600 {}` and add `.env` to `.gitignore`.",
                                        env_path.display()
                                    ),
                                    details: Some(format!("Location: {}", env_path.display())),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    (findings, scanned_env)
}

/// Detect unauthenticated / exposed local listening ports (e.g. databases, redis, docker socket)
fn check_listening_ports() -> (Vec<LaptopPostureFinding>, usize) {
    let mut findings = Vec::new();
    let mut scanned_ports = 0;

    // Check common high-risk local listening ports
    let probe_ports = [
        (
            6379,
            "Redis Server",
            AuditSeverity::High,
            "Ensure Redis requires password authentication (`requirepass`) or is bound to 127.0.0.1",
        ),
        (
            27017,
            "MongoDB Server",
            AuditSeverity::High,
            "Ensure MongoDB authentication is enabled and not exposed publicly",
        ),
        (
            9200,
            "Elasticsearch",
            AuditSeverity::High,
            "Ensure Elasticsearch security features (TLS / Basic Auth) are turned on",
        ),
        (
            2375,
            "Docker Daemon (Unencrypted TCP)",
            AuditSeverity::Critical,
            "Exposed Docker TCP port 2375 allows root privilege escalation on the host",
        ),
    ];

    for (port, service_name, severity, rec) in probe_ports {
        scanned_ports += 1;
        if std::net::TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
            std::time::Duration::from_millis(50),
        )
        .is_ok()
        {
            findings.push(LaptopPostureFinding {
                id: format!("EXPOSED_LOCAL_PORT_{}", port),
                title: format!("Active {} Listening on 127.0.0.1:{}", service_name, port),
                description: format!(
                    "Detected open local port {} ({}). Verify authentication is strictly enforced.",
                    port, service_name
                ),
                severity,
                recommendation: rec.to_string(),
                details: Some(format!("Port: {}", port)),
            });
        }
    }

    (findings, scanned_ports)
}

/// Run a complete local posture scan
pub fn run_laptop_posture_scan() -> LaptopPostureReport {
    let mut all_findings = Vec::new();

    let (ssh_findings, ssh_keys) = check_ssh_directory();
    all_findings.extend(ssh_findings);

    let (env_findings, env_files) = check_env_files();
    all_findings.extend(env_findings);

    let (port_findings, ports) = check_listening_ports();
    all_findings.extend(port_findings);

    let mut score: i32 = 100;
    for f in &all_findings {
        match f.severity {
            AuditSeverity::Critical => score -= 25,
            AuditSeverity::High => score -= 15,
            AuditSeverity::Warning => score -= 5,
            AuditSeverity::Info => score -= 1,
        }
    }
    let overall_score = score.max(0).min(100) as u32;

    LaptopPostureReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        overall_score,
        findings: all_findings,
        scanned_ssh_keys: ssh_keys,
        scanned_env_files: env_files,
        scanned_listening_ports: ports,
    }
}
