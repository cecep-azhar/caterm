use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DockerProbeSeverity {
    Info,
    Warning,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerSecurityFinding {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: DockerProbeSeverity,
    pub affected_resource: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerAuditSummary {
    pub id: String,
    pub name: String,
    pub image: String,
    pub is_privileged: bool,
    pub host_pid: bool,
    pub host_network: bool,
    pub sensitive_mounts: Vec<String>,
    pub exposed_ports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerProbeReport {
    pub timestamp: String,
    pub docker_socket_found: bool,
    pub docker_socket_path: String,
    pub socket_permissions: Option<String>,
    pub tcp_exposed_unauth: bool,
    pub containers_scanned: usize,
    pub findings: Vec<DockerSecurityFinding>,
    pub containers: Vec<ContainerAuditSummary>,
    pub security_score: u32,
}

/// SENSITIVE HOST PATHS that should never be mounted read-write into untrusted containers
const SENSITIVE_HOST_DIRECTORIES: &[&str] = &[
    "/",
    "/root",
    "/home",
    "/etc",
    "/var/run/docker.sock",
    "/run/docker.sock",
    "/proc",
    "/sys",
    "/etc/shadow",
    "/etc/passwd",
    "/etc/sudoers",
];

/// Probes docker socket permissions, potential unauthenticated daemon ports, and inspecting containers
pub fn run_docker_security_probe(
    remote_socket_info: Option<(String, u32)>, // e.g. from remote SSH command or local
    container_json_list: Option<Vec<serde_json::Value>>,
) -> DockerProbeReport {
    let now = chrono::Utc::now().to_rfc3339();
    let mut findings = Vec::new();
    let mut score: i32 = 100;
    let mut containers = Vec::new();

    let (socket_path, socket_perms) = if let Some((path, mode)) = remote_socket_info {
        (path, Some(format!("{:o}", mode & 0o777)))
    } else {
        // Local probe check
        let standard_paths = [
            "/var/run/docker.sock",
            "/run/docker.sock",
            "/run/user/1000/docker.sock",
        ];
        let mut found_path = "/var/run/docker.sock".to_string();
        let mut perms = None;

        for p in standard_paths {
            if Path::new(p).exists() {
                found_path = p.to_string();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = fs::metadata(p) {
                        let mode = meta.permissions().mode() & 0o777;
                        perms = Some(format!("{:o}", mode));
                    }
                }
                break;
            }
        }
        (found_path, perms)
    };

    let docker_socket_found = socket_perms.is_some() || Path::new(&socket_path).exists();

    // Check socket permissions (0666 / world writable is a severe vulnerability)
    if let Some(ref p) = socket_perms {
        if p.ends_with("66") || p == "666" || p == "777" {
            findings.push(DockerSecurityFinding {
                id: "DOCKER-SOCK-WORLD-RW".into(),
                title: "World-Writable Docker Socket".into(),
                description: format!("The Docker socket at {socket_path} has permissions {p}, allowing any non-root user to gain root privilege escalation."),
                severity: DockerProbeSeverity::Critical,
                affected_resource: socket_path.clone(),
                recommendation: "Restrict socket permissions: sudo chmod 660 /var/run/docker.sock && sudo chown root:docker /var/run/docker.sock".into(),
            });
            score -= 40;
        }
    }

    // Check if unauthenticated Docker TCP (2375/2376) is exposed locally
    let mut tcp_unauth = false;
    if let Ok(_stream) = std::net::TcpStream::connect_timeout(
        &"127.0.0.1:2375".parse().unwrap(),
        std::time::Duration::from_millis(200),
    ) {
        tcp_unauth = true;
        findings.push(DockerSecurityFinding {
            id: "DOCKER-TCP-UNENCRYPTED".into(),
            title: "Unencrypted & Unauthenticated Docker TCP Daemon (Port 2375)".into(),
            description: "Docker daemon is listening on plaintext TCP port 2375 without TLS/mTLS client verification. Anyone with network access can execute arbitrary code as root.".into(),
            severity: DockerProbeSeverity::Critical,
            affected_resource: "tcp://127.0.0.1:2375".into(),
            recommendation: "Disable TCP listener or configure mTLS authentication on port 2376 with client certificates.".into(),
        });
        score -= 50;
    }

    // Process parsed container representations
    if let Some(raw_containers) = container_json_list {
        for c in raw_containers {
            let id = c
                .get("Id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .chars()
                .take(12)
                .collect::<String>();
            let name = c
                .get("Names")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("unnamed")
                .trim_start_matches('/')
                .to_string();
            let image = c
                .get("Image")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let host_config = c.get("HostConfig");
            let is_privileged = host_config
                .and_then(|h| h.get("Privileged"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let host_pid = host_config
                .and_then(|h| h.get("PidMode"))
                .and_then(|v| v.as_str())
                .map(|s| s == "host")
                .unwrap_or(false);
            let host_net = host_config
                .and_then(|h| h.get("NetworkMode"))
                .and_then(|v| v.as_str())
                .map(|s| s == "host")
                .unwrap_or(false);

            let mut sensitive_mounts = Vec::new();
            if let Some(mounts) = c.get("Mounts").and_then(|v| v.as_array()) {
                for m in mounts {
                    if let Some(src) = m.get("Source").and_then(|v| v.as_str()) {
                        for sensitive in SENSITIVE_HOST_DIRECTORIES {
                            if src == *sensitive || src.starts_with(&format!("{}/", sensitive)) {
                                sensitive_mounts.push(src.to_string());
                                break;
                            }
                        }
                    }
                }
            }

            let mut exposed_ports = Vec::new();
            if let Some(ports) = c.get("Ports").and_then(|v| v.as_array()) {
                for p in ports {
                    let pub_port = p.get("PublicPort").and_then(|v| v.as_u64());
                    let priv_port = p.get("PrivatePort").and_then(|v| v.as_u64());
                    let ptype = p.get("Type").and_then(|v| v.as_str()).unwrap_or("tcp");
                    if let (Some(pub_p), Some(priv_p)) = (pub_port, priv_port) {
                        exposed_ports.push(format!("{pub_p}->{priv_p}/{ptype}"));
                    }
                }
            }

            if is_privileged {
                findings.push(DockerSecurityFinding {
                    id: format!("CONTAINER-PRIVILEGED-{id}"),
                    title: format!("Privileged Container Detected: {name}"),
                    description: format!("Container {name} ({image}) runs with --privileged=true, granting complete capability access and host device node control."),
                    severity: DockerProbeSeverity::High,
                    affected_resource: format!("Container: {name} ({id})"),
                    recommendation: "Drop --privileged flag and provide only the specific granular Linux capabilities needed (e.g. --cap-add=NET_ADMIN).".into(),
                });
                score -= 20;
            }

            if !sensitive_mounts.is_empty() {
                findings.push(DockerSecurityFinding {
                    id: format!("CONTAINER-HOST-MOUNT-{id}"),
                    title: format!("Dangerous Host Filesystem Mount: {name}"),
                    description: format!("Container {name} mounts critical host directories: {}", sensitive_mounts.join(", ")),
                    severity: DockerProbeSeverity::High,
                    affected_resource: format!("Container: {name} ({id})"),
                    recommendation: "Avoid binding root `/` or system `/etc` directories into application containers. Use dedicated named Docker volumes.".into(),
                });
                score -= 15;
            }

            containers.push(ContainerAuditSummary {
                id,
                name,
                image,
                is_privileged,
                host_pid,
                host_network: host_net,
                sensitive_mounts,
                exposed_ports,
            });
        }
    }

    let final_score = score.max(0).min(100) as u32;

    DockerProbeReport {
        timestamp: now,
        docker_socket_found,
        docker_socket_path: socket_path,
        socket_permissions: socket_perms,
        tcp_exposed_unauth: tcp_unauth,
        containers_scanned: containers.len(),
        findings,
        containers,
        security_score: final_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_docker_probe_evaluation() {
        let mock_containers = vec![
            json!({
                "Id": "a1b2c3d4e5f67890",
                "Names": ["/production-api"],
                "Image": "node:20-alpine",
                "HostConfig": {
                    "Privileged": false,
                    "PidMode": "",
                    "NetworkMode": "bridge"
                },
                "Mounts": [
                    { "Source": "/data/app", "Destination": "/app/data" }
                ],
                "Ports": [
                    { "PrivatePort": 3000, "PublicPort": 8080, "Type": "tcp" }
                ]
            }),
            json!({
                "Id": "f9e8d7c6b5a43210",
                "Names": ["/leaky-debug-tool"],
                "Image": "busybox:latest",
                "HostConfig": {
                    "Privileged": true,
                    "PidMode": "host",
                    "NetworkMode": "host"
                },
                "Mounts": [
                    { "Source": "/etc/shadow", "Destination": "/host/shadow" }
                ],
                "Ports": []
            }),
        ];

        let report = run_docker_security_probe(
            Some(("/var/run/docker.sock".into(), 0o666)),
            Some(mock_containers),
        );

        assert!(report.docker_socket_found);
        assert_eq!(report.containers_scanned, 2);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.id == "DOCKER-SOCK-WORLD-RW")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.title.contains("Privileged Container"))
        );
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.title.contains("Dangerous Host Filesystem Mount"))
        );
        assert!(report.security_score < 50);
    }
}
