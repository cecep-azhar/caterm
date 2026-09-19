//! Server Monitoring Manager (`T2-TOOL-07`).
//! Polls resource usage (CPU, RAM, Disk, Uptime) across connected active hosts via SSH commands without installing agents.
//! Results are returned as JSON to be rendered by the frontend.

use crate::error::CatermError;
use crate::ssh::SESSIONS;
use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMetrics {
    pub host_id: String,
    pub hostname: String,
    pub os_name: String,
    pub uptime: String,
    pub cpu_usage: f64,
    pub mem_total_mb: f64,
    pub mem_used_mb: f64,
    pub disk_total_gb: f64,
    pub disk_used_gb: f64,
}

fn require_non_empty(field: &str, value: &str) -> Result<(), CatermError> {
    if value.trim().is_empty() {
        return Err(CatermError::Validation(crate::error::ValidationError::Generic(format!(
            "Field {} cannot be empty",
            field
        ))));
    }
    Ok(())
}

pub fn fetch_metrics_for_host(host_id: &str) -> Result<HostMetrics, CatermError> {
    require_non_empty("host_id", host_id)?;

    let sess_arc = {
        let sessions = SESSIONS.lock().map_err(|_| CatermError::Ssh(crate::error::SshError::Generic("Lock poisoned".to_string())))?;
        // Find session handle corresponding to host_id
        if let Some(session_handle) = sessions.values().find(|h| h.host_id == host_id) {
            session_handle.session.clone()
        } else {
            return Err(CatermError::Ssh(crate::error::SshError::Generic(format!("Active session not found for host {}", host_id))));
        }
    };

    let sess_inner = sess_arc.lock().map_err(|_| CatermError::Ssh(crate::error::SshError::Generic("Lock poisoned".to_string())))?;
    let mut channel = sess_inner.channel_session().map_err(|e| {
        CatermError::Ssh(crate::error::SshError::Generic(format!("Failed to open SSH monitoring channel: {}", e)))
    })?;

    // One-liner bash script to extract metrics safely without remote dependencies
    let script = r#"
        set -e
        OS=$(uname -s)
        if [ "$OS" = "Linux" ]; then
            UPTIME=$(uptime -p | sed 's/up //')
            CPU_IDLE=$(top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print 100 - $1}')
            MEM_INFO=$(free -m | grep Mem)
            MEM_TOTAL=$(echo "$MEM_INFO" | awk '{print $2}')
            MEM_USED=$(echo "$MEM_INFO" | awk '{print $3}')
            DISK_INFO=$(df -h / | tail -1)
            DISK_TOTAL=$(echo "$DISK_INFO" | awk '{print $2}' | sed 's/G//')
            DISK_USED=$(echo "$DISK_INFO" | awk '{print $3}' | sed 's/G//')
            HOSTNAME=$(hostname)
            OS_NAME=$(cat /etc/os-release | grep "^PRETTY_NAME=" | cut -d'"' -f2 || echo "Linux")
            echo "$HOSTNAME|$OS_NAME|$UPTIME|$CPU_IDLE|$MEM_TOTAL|$MEM_USED|$DISK_TOTAL|$DISK_USED"
        elif [ "$OS" = "Darwin" ]; then
            UPTIME=$(uptime | awk -F'( |,|:)+' '{print $6 " hrs " $7 " mins"}')
            CPU_IDLE=$(top -l 1 | grep -E "^CPU" | awk '{print $7}' | sed 's/%//')
            MEM_INFO=$(vm_stat)
            MEM_TOTAL=$(sysctl -n hw.memsize | awk '{print $1 / 1024 / 1024}')
            MEM_USED=$(echo "$MEM_INFO" | grep "Pages active" | awk '{print $3}' | sed 's/\.//' | awk '{print $1 * 4096 / 1024 / 1024}')
            DISK_INFO=$(df -h / | tail -1)
            DISK_TOTAL=$(echo "$DISK_INFO" | awk '{print $2}' | sed 's/Gi//')
            DISK_USED=$(echo "$DISK_INFO" | awk '{print $3}' | sed 's/Gi//')
            HOSTNAME=$(hostname)
            OS_NAME="macOS $(sw_vers -productVersion)"
            echo "$HOSTNAME|$OS_NAME|$UPTIME|$CPU_IDLE|$MEM_TOTAL|$MEM_USED|$DISK_TOTAL|$DISK_USED"
        else
            echo "unknown|unknown|0|0|0|0|0|0"
        fi
    "#;

    channel.exec(script).map_err(|e| {
        CatermError::Ssh(crate::error::SshError::Generic(format!("Failed to execute monitoring script: {}", e)))
    })?;

    let mut output = String::new();
    channel.read_to_string(&mut output).unwrap_or_default();
    channel.wait_close().unwrap_or_default();

    let output = output.trim();
    let parts: Vec<&str> = output.split('|').collect();

    if parts.len() < 8 {
        return Err(CatermError::Ssh(crate::error::SshError::Generic(format!("Invalid metrics response: {}", output))));
    }

    Ok(HostMetrics {
        host_id: host_id.to_string(),
        hostname: parts[0].to_string(),
        os_name: parts[1].to_string(),
        uptime: parts[2].to_string(),
        cpu_usage: parts[3].parse::<f64>().unwrap_or(0.0),
        mem_total_mb: parts[4].parse::<f64>().unwrap_or(0.0),
        mem_used_mb: parts[5].parse::<f64>().unwrap_or(0.0),
        disk_total_gb: parts[6].parse::<f64>().unwrap_or(0.0),
        disk_used_gb: parts[7].parse::<f64>().unwrap_or(0.0),
    })
}

pub fn poll_active_metrics() -> Result<Vec<HostMetrics>, CatermError> {
    let active_host_ids: Vec<String> = {
        let sessions = SESSIONS.lock().map_err(|_| CatermError::Ssh(crate::error::SshError::Generic("Lock poisoned".to_string())))?;
        let mut ids: Vec<String> = sessions.values().map(|m| m.host_id.clone()).collect();
        ids.sort();
        ids.dedup();
        ids
    };

    let mut results = Vec::new();
    for hid in active_host_ids.into_iter() {
        if let Ok(metrics) = fetch_metrics_for_host(&hid) {
            results.push(metrics);
        }
    }

    Ok(results)
}
