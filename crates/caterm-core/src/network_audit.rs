//! DevOps Network & Security Audit Tooling (`network_audit.rs`).
//!
//! Provides automated, agentless remote audits over SSH exec sessions:
//! 1. Egress Port & Hole Hunter: tests outbound connectivity to critical/dangerous ports
//!    (25 SMTP, 53 DNS leak, 445 SMB, 3306 MySQL, 5432 Postgres, 6379 Redis).
//! 2. Inbound Exposure Audit: detects listening ports (`ss -tulpn` / `netstat`) binding to `0.0.0.0` or `::`
//!    without authentication or internal isolation.
//! 3. Network Benchmark & Latency Probe: MTR/traceroute hop analysis, ping latency, and packet loss visualizer.
//! 4. SSH & Host Hardening Checklist: verifies `/etc/ssh/sshd_config` (PermitRootLogin, PasswordAuthentication),
//!    firewall status (UFW / iptables), and kernel security baselines.

use crate::error::CatermError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EgressPortResult {
    pub port: u16,
    pub target: String,
    pub service_label: String,
    pub status: String,     // "BLOCKED", "OPEN_LEAK", "TIMEOUT", "SKIPPED"
    pub risk_level: String, // "CRITICAL", "HIGH", "MEDIUM", "LOW"
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundListeningPort {
    pub proto: String,
    pub local_address: String,
    pub port: u16,
    pub process: String,
    pub is_wildcard: bool, // 0.0.0.0 or [::]
    pub risk_level: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatencyProbeResult {
    pub target: String,
    pub packet_loss_pct: f64,
    pub avg_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub hops: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardeningCheckItem {
    pub name: String,
    pub category: String, // "SSH", "FIREWALL", "KERNEL"
    pub current_value: String,
    pub recommended_value: String,
    pub status: String, // "PASS", "WARN", "FAIL"
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditReport {
    pub host_id: String,
    pub timestamp: i64,
    pub egress_results: Vec<EgressPortResult>,
    pub inbound_ports: Vec<InboundListeningPort>,
    pub latency_probe: LatencyProbeResult,
    pub hardening_checklist: Vec<HardeningCheckItem>,
    pub summary_score: u32, // 0 - 100%
}

/// Bash script executed on remote host to gather all audit information in a structured, parseable format.
const AUDIT_SCRIPT: &str = r#"
echo "===CATERM_AUDIT_START==="

echo "---SECTION:EGRESS---"
# Test egress ports using bash /dev/tcp or nc/curl with strict 2s timeout
test_egress() {
    port=$1
    host=$2
    label=$3
    risk=$4
    timeout 2 bash -c "</dev/tcp/$host/$port" 2>/dev/null
    res=$?
    if [ $res -eq 0 ]; then
        echo "$port|$host|$label|OPEN_LEAK|$risk|Outbound connection succeeded"
    else
        echo "$port|$host|$label|BLOCKED|$risk|Connection blocked or filtered"
    fi
}

test_egress 25 "smtp.gmail.com" "SMTP Mail" "HIGH"
test_egress 53 "1.1.1.1" "DNS Leak" "MEDIUM"
test_egress 445 "smb.example.com" "SMB NetBIOS" "CRITICAL"
test_egress 3306 "mysql.example.com" "MySQL Remote" "HIGH"
test_egress 5432 "postgres.example.com" "Postgres Remote" "HIGH"
test_egress 6379 "redis.example.com" "Redis Unauth" "CRITICAL"

echo "---SECTION:INBOUND---"
# Check listening ports
if command -v ss >/dev/null 2>&1; then
    ss -tulpn | tail -n +2 | while read -r line; do
        proto=$(echo "$line" | awk '{print $1}')
        local_addr=$(echo "$line" | awk '{print $5}')
        proc=$(echo "$line" | awk '{print $7}')
        echo "$proto|$local_addr|$proc"
    done
elif command -v netstat >/dev/null 2>&1; then
    netstat -tulpn 2>/dev/null | tail -n +3 | while read -r line; do
        proto=$(echo "$line" | awk '{print $1}')
        local_addr=$(echo "$line" | awk '{print $4}')
        proc=$(echo "$line" | awk '{print $7}')
        echo "$proto|$local_addr|$proc"
    done
fi

echo "---SECTION:PROBE---"
# Latency & ping check to 1.1.1.1
if command -v ping >/dev/null 2>&1; then
    ping_out=$(ping -c 4 -W 2 1.1.1.1 2>/dev/null || true)
    echo "$ping_out"
fi
echo "---HOPS---"
if command -v traceroute >/dev/null 2>&1; then
    traceroute -m 8 -q 1 1.1.1.1 2>/dev/null || true
elif command -v tracepath >/dev/null 2>&1; then
    tracepath -m 8 1.1.1.1 2>/dev/null || true
fi

echo "---SECTION:HARDENING---"
# SSH configs
sshd_file="/etc/ssh/sshd_config"
if [ -f "$sshd_file" ]; then
    root_login=$(grep -Ei '^\s*PermitRootLogin' "$sshd_file" | tail -1 | awk '{print $2}' || true)
    pw_auth=$(grep -Ei '^\s*PasswordAuthentication' "$sshd_file" | tail -1 | awk '{print $2}' || true)
    pubkey_auth=$(grep -Ei '^\s*PubkeyAuthentication' "$sshd_file" | tail -1 | awk '{print $2}' || true)
    echo "PermitRootLogin|${root_login:-default_yes}"
    echo "PasswordAuthentication|${pw_auth:-default_yes}"
    echo "PubkeyAuthentication|${pubkey_auth:-default_yes}"
else
    echo "PermitRootLogin|unknown"
    echo "PasswordAuthentication|unknown"
    echo "PubkeyAuthentication|unknown"
fi

# Firewall check
if command -v ufw >/dev/null 2>&1; then
    ufw_stat=$(ufw status 2>/dev/null | head -1 || echo "unknown")
    echo "UFWStatus|$ufw_stat"
elif command -v iptables >/dev/null 2>&1; then
    rules_count=$(iptables -L -n 2>/dev/null | wc -l || echo "0")
    echo "IptablesRulesCount|$rules_count"
else
    echo "Firewall|none"
fi

echo "===CATERM_AUDIT_END==="
"#;

pub fn run_network_security_audit(host_id: &str) -> Result<AuditReport, CatermError> {
    if host_id.trim().is_empty() {
        return Err(CatermError::Validation(
            crate::error::ValidationError::Generic("host_id cannot be empty".to_string()),
        ));
    }

    let raw_output = if host_id == "local" || host_id == "__local__" {
        #[cfg(not(windows))]
        {
            let mut cmd = std::process::Command::new("sh");
            cmd.args(["-c", AUDIT_SCRIPT]);
            let out = cmd.output().map_err(|e| {
                CatermError::Io(crate::error::IoError::Generic(format!(
                    "Failed to execute local audit script: {e}"
                )))
            })?;
            String::from_utf8_lossy(&out.stdout).to_string()
        }
        #[cfg(windows)]
        {
            return Err(CatermError::Validation(
                crate::error::ValidationError::Generic(
                    "Local audit on Windows is not supported, run against Linux SSH host"
                        .to_string(),
                ),
            ));
        }
    } else {
        use std::io::Read;
        crate::ssh::with_exec_session(host_id, |sess| {
            let mut channel = sess.channel_session().map_err(|e| {
                CatermError::Ssh(crate::error::SshError::Generic(format!(
                    "Failed to open SSH audit channel: {e}"
                )))
            })?;
            channel.exec(AUDIT_SCRIPT).map_err(|e| {
                CatermError::Ssh(crate::error::SshError::Generic(format!(
                    "Failed to execute audit script: {e}"
                )))
            })?;

            let mut buf = Vec::new();
            let _ = channel.read_to_end(&mut buf);
            let _ = channel.wait_close();
            Ok(String::from_utf8_lossy(&buf).to_string())
        })?
    };

    let report = parse_audit_output(host_id, &raw_output);

    // Record audit event in Caterm command logs
    let _ = crate::audit::log_event(
        "SECURITY_AUDIT",
        Some(host_id),
        &format!(
            "Completed Network & Security Audit for host {host_id}. Score: {}%",
            report.summary_score
        ),
    );

    Ok(report)
}

pub(crate) fn parse_audit_output(host_id: &str, raw: &str) -> AuditReport {
    let now = chrono::Utc::now().timestamp_millis();
    let mut egress_results = Vec::new();
    let mut inbound_ports = Vec::new();
    let mut hardening_checklist = Vec::new();
    let mut ping_lines = Vec::new();
    let mut hop_lines = Vec::new();

    let mut current_section = "";

    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with("---SECTION:EGRESS---") {
            current_section = "EGRESS";
            continue;
        } else if line.starts_with("---SECTION:INBOUND---") {
            current_section = "INBOUND";
            continue;
        } else if line.starts_with("---SECTION:PROBE---") {
            current_section = "PROBE";
            continue;
        } else if line.starts_with("---HOPS---") {
            current_section = "HOPS";
            continue;
        } else if line.starts_with("---SECTION:HARDENING---") {
            current_section = "HARDENING";
            continue;
        } else if line.starts_with("===CATERM_AUDIT_") {
            continue;
        }

        if line.is_empty() {
            continue;
        }

        match current_section {
            "EGRESS" => {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 6 {
                    let port = parts
                        .first()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or(0);
                    let target = parts.get(1).unwrap_or(&"").to_string();
                    let service_label = parts.get(2).unwrap_or(&"").to_string();
                    let status = parts.get(3).unwrap_or(&"BLOCKED").to_string();
                    let risk_level = parts.get(4).unwrap_or(&"MEDIUM").to_string();
                    let notes = parts.get(5).unwrap_or(&"").to_string();

                    egress_results.push(EgressPortResult {
                        port,
                        target,
                        service_label,
                        status,
                        risk_level,
                        notes,
                    });
                }
            }
            "INBOUND" => {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 3 {
                    let proto = parts.first().unwrap_or(&"tcp").to_string();
                    let full_addr = parts.get(1).unwrap_or(&"").to_string();
                    let proc = parts.get(2).unwrap_or(&"-").to_string();

                    let (addr, port) = parse_addr_and_port(&full_addr);
                    let is_wildcard =
                        addr == "0.0.0.0" || addr == "::" || addr == "*" || addr.is_empty();

                    let risk_level = if is_wildcard && (port == 22 || port == 80 || port == 443) {
                        "LOW".to_string()
                    } else if is_wildcard
                        && (port == 3306
                            || port == 5432
                            || port == 6379
                            || port == 27017
                            || port == 9200)
                    {
                        "CRITICAL".to_string()
                    } else if is_wildcard {
                        "HIGH".to_string()
                    } else {
                        "LOW".to_string()
                    };

                    let recommendation = if risk_level == "CRITICAL" {
                        "Bind to 127.0.0.1 or protect with firewall/UFW rules immediately."
                            .to_string()
                    } else if is_wildcard {
                        "Exposed to all interfaces (0.0.0.0). Verify if public exposure is required.".to_string()
                    } else {
                        "Protected / Localhost binding only.".to_string()
                    };

                    inbound_ports.push(InboundListeningPort {
                        proto,
                        local_address: full_addr,
                        port,
                        process: proc,
                        is_wildcard,
                        risk_level,
                        recommendation,
                    });
                }
            }
            "PROBE" => {
                ping_lines.push(line.to_string());
            }
            "HOPS" => {
                hop_lines.push(line.to_string());
            }
            "HARDENING" => {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 2 {
                    let key = parts.first().unwrap_or(&"");
                    let val = parts.get(1).unwrap_or(&"");

                    match *key {
                        "PermitRootLogin" => {
                            let pass = val.eq_ignore_ascii_case("no")
                                || val.eq_ignore_ascii_case("prohibit-password");
                            hardening_checklist.push(HardeningCheckItem {
                                name: "SSH PermitRootLogin".to_string(),
                                category: "SSH".to_string(),
                                current_value: val.to_string(),
                                recommended_value: "no / prohibit-password".to_string(),
                                status: if pass {
                                    "PASS".to_string()
                                } else {
                                    "FAIL".to_string()
                                },
                                details: if pass {
                                    "Root login over SSH is restricted.".to_string()
                                } else {
                                    "Direct root SSH logins permitted. High security risk."
                                        .to_string()
                                },
                            });
                        }
                        "PasswordAuthentication" => {
                            let pass = val.eq_ignore_ascii_case("no");
                            hardening_checklist.push(HardeningCheckItem {
                                name: "SSH PasswordAuthentication".to_string(),
                                category: "SSH".to_string(),
                                current_value: val.to_string(),
                                recommended_value: "no".to_string(),
                                status: if pass {
                                    "PASS".to_string()
                                } else {
                                    "WARN".to_string()
                                },
                                details: if pass {
                                    "Password auth disabled; keys required.".to_string()
                                } else {
                                    "Password authentication is active; consider SSH keys only."
                                        .to_string()
                                },
                            });
                        }
                        "PubkeyAuthentication" => {
                            let pass = !val.eq_ignore_ascii_case("no");
                            hardening_checklist.push(HardeningCheckItem {
                                name: "SSH PubkeyAuthentication".to_string(),
                                category: "SSH".to_string(),
                                current_value: val.to_string(),
                                recommended_value: "yes".to_string(),
                                status: if pass {
                                    "PASS".to_string()
                                } else {
                                    "FAIL".to_string()
                                },
                                details: if pass {
                                    "Public key authentication is enabled.".to_string()
                                } else {
                                    "Public key authentication is disabled.".to_string()
                                },
                            });
                        }
                        "UFWStatus" => {
                            let pass = val.to_lowercase().contains("active");
                            hardening_checklist.push(HardeningCheckItem {
                                name: "UFW Firewall Status".to_string(),
                                category: "FIREWALL".to_string(),
                                current_value: val.to_string(),
                                recommended_value: "active".to_string(),
                                status: if pass {
                                    "PASS".to_string()
                                } else {
                                    "FAIL".to_string()
                                },
                                details: if pass {
                                    "UFW firewall is enabled and active.".to_string()
                                } else {
                                    "UFW firewall is inactive or disabled.".to_string()
                                },
                            });
                        }
                        "IptablesRulesCount" => {
                            let count = val.parse::<i32>().unwrap_or(0);
                            let pass = count > 8;
                            hardening_checklist.push(HardeningCheckItem {
                                name: "Iptables Rules Present".to_string(),
                                category: "FIREWALL".to_string(),
                                current_value: format!("{count} rules"),
                                recommended_value: "> 0 filtering rules".to_string(),
                                status: if pass {
                                    "PASS".to_string()
                                } else {
                                    "WARN".to_string()
                                },
                                details: if pass {
                                    "Iptables rules detected in system.".to_string()
                                } else {
                                    "Minimal or no iptables filtering rules found.".to_string()
                                },
                            });
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // Parse ping probe
    let (loss_pct, avg_lat, min_lat, max_lat) = parse_ping_summary(&ping_lines);

    let latency_probe = LatencyProbeResult {
        target: "1.1.1.1 (Cloudflare DNS)".to_string(),
        packet_loss_pct: loss_pct,
        avg_latency_ms: avg_lat,
        min_latency_ms: min_lat,
        max_latency_ms: max_lat,
        hops: hop_lines,
        status: if loss_pct == 0.0 && avg_lat > 0.0 {
            "EXCELLENT".to_string()
        } else if loss_pct < 20.0 {
            "FAIR".to_string()
        } else {
            "DEGRADED".to_string()
        },
    };

    // Calculate score
    let mut total_score: i32 = 100;
    for e in &egress_results {
        if e.status == "OPEN_LEAK" && e.risk_level == "CRITICAL" {
            total_score -= 15;
        } else if e.status == "OPEN_LEAK" && e.risk_level == "HIGH" {
            total_score -= 10;
        }
    }

    for p in &inbound_ports {
        if p.risk_level == "CRITICAL" {
            total_score -= 20;
        } else if p.risk_level == "HIGH" {
            total_score -= 8;
        }
    }

    for h in &hardening_checklist {
        if h.status == "FAIL" {
            total_score -= 12;
        } else if h.status == "WARN" {
            total_score -= 5;
        }
    }

    let summary_score = total_score.clamp(0, 100) as u32;

    AuditReport {
        host_id: host_id.to_string(),
        timestamp: now,
        egress_results,
        inbound_ports,
        latency_probe,
        hardening_checklist,
        summary_score,
    }
}

fn parse_addr_and_port(addr_str: &str) -> (String, u16) {
    let s = addr_str.trim();
    if let Some(idx) = s.rfind(':') {
        let (ip_part, port_part) = s.split_at(idx);
        let port = port_part
            .trim_start_matches(':')
            .parse::<u16>()
            .unwrap_or(0);
        let ip = ip_part.trim_matches(|c| c == '[' || c == ']').to_string();
        (ip, port)
    } else {
        (s.to_string(), 0)
    }
}

fn parse_ping_summary(lines: &[String]) -> (f64, f64, f64, f64) {
    let mut loss = 0.0;
    let mut avg = 0.0;
    let mut min = 0.0;
    let mut max = 0.0;

    for line in lines {
        if line.contains("% packet loss") {
            if let Some(idx) = line.find('%') {
                let start = line[..idx].rfind(' ').map(|i| i + 1).unwrap_or(0);
                loss = line[start..idx].parse::<f64>().unwrap_or(0.0);
            }
        }
        if line.starts_with("rtt ") || line.starts_with("round-trip ") {
            if let Some(eq_idx) = line.find('=') {
                let stats = line[eq_idx + 1..].trim();
                let parts: Vec<&str> = stats.split('/').collect();
                if parts.len() >= 3 {
                    min = parts
                        .first()
                        .and_then(|p| p.trim().parse::<f64>().ok())
                        .unwrap_or(0.0);
                    avg = parts
                        .get(1)
                        .and_then(|p| p.trim().parse::<f64>().ok())
                        .unwrap_or(0.0);
                    max = parts
                        .get(2)
                        .and_then(|p| p.trim().parse::<f64>().ok())
                        .unwrap_or(0.0);
                }
            }
        }
    }

    (loss, avg, min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_audit_output_checks_egress_and_ports() {
        let sample = r#"
===CATERM_AUDIT_START===
---SECTION:EGRESS---
25|smtp.gmail.com|SMTP Mail|BLOCKED|HIGH|Connection blocked or filtered
6379|redis.example.com|Redis Unauth|OPEN_LEAK|CRITICAL|Outbound connection succeeded
---SECTION:INBOUND---
tcp|0.0.0.0:22|sshd
tcp|0.0.0.0:6379|redis-server
tcp|127.0.0.1:3306|mysqld
---SECTION:PROBE---
4 packets transmitted, 4 received, 0% packet loss, time 3004ms
rtt min/avg/max/mdev = 12.3/15.8/20.1/3.2 ms
---HOPS---
1 192.168.1.1 1.2ms
2 10.0.0.1 5.4ms
---SECTION:HARDENING---
PermitRootLogin|no
PasswordAuthentication|yes
PubkeyAuthentication|yes
UFWStatus|Status: active
===CATERM_AUDIT_END===
"#;

        let report = parse_audit_output("test-host", sample);
        assert_eq!(report.host_id, "test-host");
        assert_eq!(report.egress_results.len(), 2);
        assert_eq!(report.egress_results[1].status, "OPEN_LEAK");
        assert_eq!(report.egress_results[1].risk_level, "CRITICAL");

        assert_eq!(report.inbound_ports.len(), 3);
        assert_eq!(report.inbound_ports[1].risk_level, "CRITICAL");
        assert_eq!(report.inbound_ports[2].risk_level, "LOW");

        assert_eq!(report.latency_probe.packet_loss_pct, 0.0);
        assert_eq!(report.latency_probe.avg_latency_ms, 15.8);

        assert_eq!(report.hardening_checklist.len(), 4);
        assert_eq!(report.hardening_checklist[0].status, "PASS");
        assert_eq!(report.hardening_checklist[1].status, "WARN");
        assert_eq!(report.hardening_checklist[3].status, "PASS");
    }
}
