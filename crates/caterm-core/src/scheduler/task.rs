//! Scheduled Tasks and Cron Engine data models (REQ-38).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    SshCommand,
    SftpBackup,
    LocalScript,
}

impl TaskType {
    /// # Infallible — maps enum variant to static str representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SshCommand => "ssh_command",
            Self::SftpBackup => "sftp_backup",
            Self::LocalScript => "local_script",
        }
    }

    /// # Infallible — parses enum variant from string with fallback to SshCommand.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sftp_backup" => Self::SftpBackup,
            "local_script" => Self::LocalScript,
            _ => Self::SshCommand,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Success,
    Failed,
    Running,
    Timeout,
    Cancelled,
}

impl TaskStatus {
    /// # Infallible — maps enum variant to static str representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Running => "running",
            Self::Timeout => "timeout",
            Self::Cancelled => "cancelled",
        }
    }

    /// # Infallible — parses enum variant from string with fallback to Failed.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "success" => Self::Success,
            "failed" => Self::Failed,
            "running" => Self::Running,
            "timeout" => Self::Timeout,
            "cancelled" => Self::Cancelled,
            _ => Self::Failed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTaskRecord {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub task_type: TaskType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_id: Option<String>,
    pub schedule_expr: String,
    pub is_enabled: bool,

    // SSH Command Configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_script: Option<String>,
    pub timeout_seconds: u32,

    // SFTP Backup Configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_src_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_dest_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_pre_cmd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_post_cmd: Option<String>,
    pub retention_count: u32,

    // Notification & Status
    pub notify_on_success: bool,
    pub notify_on_failure: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_status: Option<TaskStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_run_at: Option<String>,

    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub task_type: TaskType,
    pub host_id: Option<String>,
    pub schedule_expr: String,
    #[serde(default = "default_true")]
    pub is_enabled: bool,

    // SSH Command Configuration
    pub command_script: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,

    // SFTP Backup Configuration
    pub remote_src_path: Option<String>,
    pub local_dest_dir: Option<String>,
    pub remote_pre_cmd: Option<String>,
    pub remote_post_cmd: Option<String>,
    #[serde(default = "default_retention")]
    pub retention_count: u32,

    // Notification
    #[serde(default)]
    pub notify_on_success: bool,
    #[serde(default = "default_true")]
    pub notify_on_failure: bool,
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u32 {
    300
}

fn default_retention() -> u32 {
    7
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskExecutionLog {
    pub id: String,
    pub task_id: String,
    pub started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    pub status: TaskStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes_transferred: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRunResult {
    pub log: TaskExecutionLog,
    pub success: bool,
}
