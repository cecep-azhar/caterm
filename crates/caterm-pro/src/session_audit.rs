use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionAuditHeader {
    pub version: u32,
    pub width: u32,
    pub height: u32,
    pub timestamp: i64,
    pub title: String,
    pub session_id: String,
    pub user: String,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditBlockReceipt {
    pub session_id: String,
    pub total_chunks: usize,
    pub total_bytes: usize,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub genesis_hash: String,
    pub terminal_hash: String,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CastChunk {
    pub time_offset: f64,
    pub event_type: String, // "o" for output, "i" for input
    pub data: String,
    pub prev_hash: String,
    pub block_hash: String,
}

#[derive(Debug, Clone)]
pub struct SessionAuditRecorder {
    pub session_id: String,
    pub header: SessionAuditHeader,
    pub prev_hash: String,
    pub chunks: Vec<CastChunk>,
    pub total_bytes: usize,
}

impl SessionAuditRecorder {
    pub fn new(session_id: String, user: String, host: String, width: u32, height: u32) -> Self {
        let now = Utc::now().timestamp();
        let genesis_seed = format!("caterm-audit-genesis-{session_id}-{now}");
        let mut hasher = Sha256::new();
        hasher.update(genesis_seed.as_bytes());
        let genesis_hash = hex::encode(hasher.finalize());

        let header = SessionAuditHeader {
            version: 2,
            width,
            height,
            timestamp: now,
            title: format!("CATerm Audit Log [{session_id}]"),
            session_id: session_id.clone(),
            user,
            host,
        };

        SessionAuditRecorder {
            session_id,
            header,
            prev_hash: genesis_hash,
            chunks: Vec::new(),
            total_bytes: 0,
        }
    }

    /// Appends a new terminal output/input event with cryptographic hash chaining
    pub fn record_event(&mut self, time_offset: f64, event_type: &str, data: &str) -> CastChunk {
        let mut hasher = Sha256::new();
        hasher.update(self.prev_hash.as_bytes());
        hasher.update(time_offset.to_string().as_bytes());
        hasher.update(event_type.as_bytes());
        hasher.update(data.as_bytes());
        let block_hash = hex::encode(hasher.finalize());

        let chunk = CastChunk {
            time_offset,
            event_type: event_type.to_string(),
            data: data.to_string(),
            prev_hash: self.prev_hash.clone(),
            block_hash: block_hash.clone(),
        };

        self.prev_hash = block_hash;
        self.total_bytes += data.len();
        self.chunks.push(chunk.clone());
        chunk
    }

    /// Exports session as standard asciinema `.cast` stream format
    pub fn export_cast_format(&self) -> String {
        let header_json = serde_json::to_string(&self.header).unwrap_or_default();
        let mut lines = vec![header_json];

        for c in &self.chunks {
            let row = serde_json::json!([c.time_offset, c.event_type, c.data]);
            lines.push(row.to_string());
        }

        lines.join("\n")
    }

    /// Generates verifiable cryptographic receipt
    pub fn generate_receipt(&self) -> AuditBlockReceipt {
        let now = Utc::now().timestamp();
        let genesis = self
            .chunks
            .first()
            .map(|c| c.prev_hash.clone())
            .unwrap_or_else(|| self.prev_hash.clone());

        AuditBlockReceipt {
            session_id: self.session_id.clone(),
            total_chunks: self.chunks.len(),
            total_bytes: self.total_bytes,
            start_timestamp: self.header.timestamp,
            end_timestamp: now,
            genesis_hash: genesis,
            terminal_hash: self.prev_hash.clone(),
            is_valid: true,
        }
    }
}

/// Verifies whether an array of chunks forms an unbroken SHA-256 cryptographic chain
pub fn verify_session_audit_chain(genesis_hash: &str, chunks: &[CastChunk]) -> bool {
    let mut current_hash = genesis_hash.to_string();

    for chunk in chunks {
        if chunk.prev_hash != current_hash {
            return false;
        }

        let mut hasher = Sha256::new();
        hasher.update(current_hash.as_bytes());
        hasher.update(chunk.time_offset.to_string().as_bytes());
        hasher.update(chunk.event_type.as_bytes());
        hasher.update(chunk.data.as_bytes());
        let expected = hex::encode(hasher.finalize());

        if chunk.block_hash != expected {
            return false;
        }
        current_hash = chunk.block_hash.clone();
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_audit_hash_chaining() {
        let mut recorder = SessionAuditRecorder::new(
            "sess-test-99".into(),
            "root".into(),
            "192.168.1.50".into(),
            120,
            40,
        );

        let c1 = recorder.record_event(0.12, "o", "Welcome to Ubuntu 24.04\n");
        let _c2 = recorder.record_event(1.45, "o", "root@prod:~# ls -la\n");
        let c3 = recorder.record_event(2.10, "o", "total 48\ndrwxr-xr-x 4 root root 4096\n");

        let receipt = recorder.generate_receipt();
        assert_eq!(receipt.total_chunks, 3);
        assert_eq!(receipt.terminal_hash, c3.block_hash);

        // Verify valid chain
        assert!(verify_session_audit_chain(&c1.prev_hash, &recorder.chunks));

        // Tamper test: modify c2 data
        let mut tampered_chunks = recorder.chunks.clone();
        tampered_chunks[1].data = "root@prod:~# rm -rf /\n".into();
        assert!(!verify_session_audit_chain(&c1.prev_hash, &tampered_chunks));
    }
}
