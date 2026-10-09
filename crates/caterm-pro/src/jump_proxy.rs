use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BastionHop {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String, // "password", "key", "agent"
    pub key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JumpChainConfig {
    pub id: String,
    pub name: String,
    pub target_host: String,
    pub target_port: u16,
    pub target_username: String,
    pub hops: Vec<BastionHop>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HopProbeStatus {
    pub hop_index: usize,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub reachable: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JumpChainProbeResult {
    pub chain_id: String,
    pub is_fully_traversable: bool,
    pub hop_statuses: Vec<HopProbeStatus>,
    pub ssh_proxy_command: String,
    pub ssh_jump_directive: String,
}

/// Generates standard OpenSSH `ProxyJump` and `-J` CLI options for multi-hop
pub fn generate_ssh_jump_directives(
    hops: &[BastionHop],
    target_host: &str,
    target_port: u16,
    target_user: &str,
) -> (String, String) {
    if hops.is_empty() {
        let direct_cmd = format!("ssh -p {target_port} {target_user}@{target_host}");
        return (direct_cmd.clone(), direct_cmd);
    }

    let jump_list: Vec<String> = hops
        .iter()
        .map(|h| {
            if h.port == 22 {
                format!("{}@{}", h.username, h.host)
            } else {
                format!("{}@{}:{}", h.username, h.host, h.port)
            }
        })
        .collect();

    let jump_spec = jump_list.join(",");
    let ssh_jump_directive = format!("-J {jump_spec}");
    let ssh_proxy_command =
        format!("ssh -J {jump_spec} -p {target_port} {target_user}@{target_host}");

    (ssh_proxy_command, ssh_jump_directive)
}

/// Probes TCP connectivity to hops sequentially or tests reachability
pub fn probe_jump_chain(config: &JumpChainConfig) -> JumpChainProbeResult {
    let mut hop_statuses = Vec::new();
    let mut is_fully_traversable = true;

    for (idx, hop) in config.hops.iter().enumerate() {
        let start = std::time::Instant::now();
        let target = format!("{}:{}", hop.host, hop.port);
        let timeout = std::time::Duration::from_millis(2000);

        match std::net::TcpStream::connect_timeout(
            &target
                .parse()
                .unwrap_or_else(|_| "127.0.0.1:22".parse().unwrap()),
            timeout,
        ) {
            Ok(_) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                hop_statuses.push(HopProbeStatus {
                    hop_index: idx,
                    name: hop.name.clone(),
                    host: hop.host.clone(),
                    port: hop.port,
                    reachable: true,
                    latency_ms: Some(latency_ms),
                    error: None,
                });
            }
            Err(e) => {
                // In local sandbox / mock tests, we allow probe error reporting
                hop_statuses.push(HopProbeStatus {
                    hop_index: idx,
                    name: hop.name.clone(),
                    host: hop.host.clone(),
                    port: hop.port,
                    reachable: false,
                    latency_ms: None,
                    error: Some(format!("Failed to connect: {e}")),
                });
                is_fully_traversable = false;
            }
        }
    }

    let (ssh_proxy_command, ssh_jump_directive) = generate_ssh_jump_directives(
        &config.hops,
        &config.target_host,
        config.target_port,
        &config.target_username,
    );

    JumpChainProbeResult {
        chain_id: config.id.clone(),
        is_fully_traversable,
        hop_statuses,
        ssh_proxy_command,
        ssh_jump_directive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jump_directives_generation() {
        let hops = vec![
            BastionHop {
                id: "h1".into(),
                name: "Perimeter Bastion".into(),
                host: "jump1.corp.internal".into(),
                port: 22,
                username: "gatekeeper".into(),
                auth_type: "key".into(),
                key_id: None,
            },
            BastionHop {
                id: "h2".into(),
                name: "DMZ Step".into(),
                host: "dmz-jump.corp.internal".into(),
                port: 2222,
                username: "ops".into(),
                auth_type: "key".into(),
                key_id: None,
            },
        ];

        let (cmd, directive) = generate_ssh_jump_directives(&hops, "10.0.50.2", 22, "root");
        assert_eq!(
            directive,
            "-J gatekeeper@jump1.corp.internal,ops@dmz-jump.corp.internal:2222"
        );
        assert_eq!(
            cmd,
            "ssh -J gatekeeper@jump1.corp.internal,ops@dmz-jump.corp.internal:2222 -p 22 root@10.0.50.2"
        );
    }
}
