//! Master AI Routing, Custom Context & Persistent Habit Memory Engine for CATerm.
//! Combines custom personas, zero-knowledge SQLite/FTS5 habit memory, multi-provider model routing,
//! privacy scrubbing, and automated execution plans.

pub mod context_builder;
pub mod db;
pub mod dispatcher;
pub mod habit_learner;
pub mod models;
pub mod scrubber;

pub use context_builder::*;
pub use db::*;
pub use dispatcher::*;
pub use habit_learner::*;
pub use models::*;
pub use scrubber::*;

use crate::error::{AiError, CatermError, DbError, SshError, ValidationError};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// High-Level Public API for Tauri IPC Commands
// ---------------------------------------------------------------------------

pub fn get_all_providers() -> Result<Vec<AiProviderSummary>, CatermError> {
    let conn = crate::db::open()?;
    let configs = db::get_providers(&conn)?;
    Ok(configs
        .into_iter()
        .map(|c| AiProviderSummary {
            id: c.id,
            name: c.name,
            provider_type: c.provider_type,
            base_url: c.base_url,
            default_model: c.default_model,
            is_active: c.is_active,
            has_api_key: c.api_key.as_ref().map(|k| !k.trim().is_empty()).unwrap_or(false),
            created_at: c.created_at,
            updated_at: c.updated_at,
        })
        .collect())
}

pub fn save_ai_provider(provider: AiProviderConfig) -> Result<(), CatermError> {
    let conn = crate::db::open()?;
    db::save_provider(&conn, &provider)
}

pub fn delete_ai_provider(id: &str) -> Result<(), CatermError> {
    let conn = crate::db::open()?;
    db::delete_provider(&conn, id)
}

pub fn get_task_routing_matrix() -> Result<Vec<TaskRouteRule>, CatermError> {
    let conn = crate::db::open()?;
    db::get_routing_matrix(&conn)
}

pub fn save_task_routing_rule(rule: TaskRouteRule) -> Result<(), CatermError> {
    let conn = crate::db::open()?;
    db::save_routing_rule(&conn, &rule)
}

pub fn get_user_personas() -> Result<Vec<SystemPersona>, CatermError> {
    let conn = crate::db::open()?;
    db::get_personas(&conn)
}

pub fn save_user_persona(persona: SystemPersona) -> Result<(), CatermError> {
    let conn = crate::db::open()?;
    db::save_persona(&conn, &persona)
}

pub fn delete_user_persona(id: &str) -> Result<(), CatermError> {
    let conn = crate::db::open()?;
    db::delete_persona(&conn, id)
}

pub fn get_habit_memories() -> Result<Vec<HabitFact>, CatermError> {
    let conn = crate::db::open()?;
    db::get_habits(&conn)
}

pub fn search_habit_memories(query: &str, limit: usize) -> Result<Vec<HabitFact>, CatermError> {
    let conn = crate::db::open()?;
    db::search_relevant_habits(&conn, query, limit)
}

pub fn toggle_habit_pin_status(id: &str) -> Result<bool, CatermError> {
    let conn = crate::db::open()?;
    db::toggle_habit_pin(&conn, id)
}

pub fn delete_habit_memory(id: &str) -> Result<(), CatermError> {
    let conn = crate::db::open()?;
    db::delete_habit(&conn, id)
}

pub fn dispatch_task(
    task_type: TaskType,
    user_query: &str,
    terminal_ctx: Option<TerminalContext>,
) -> Result<DispatchResult, CatermError> {
    let conn = crate::db::open()?;
    let dispatcher = Dispatcher::new(&conn);
    dispatcher.dispatch(task_type, user_query, terminal_ctx.as_ref())
}

// ---------------------------------------------------------------------------
// Existing Plan / Chat / Settings Data Structs & Helpers for Backwards Compat
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiPlanStep {
    pub step: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub title: String,
    pub command: String,
    pub description: String,
    #[serde(alias = "isDangerous", alias = "isDanger")]
    pub is_dangerous: bool,
    #[serde(default, alias = "isSudo", skip_serializing_if = "Option::is_none")]
    pub is_sudo: Option<bool>,
    #[serde(default, alias = "stepNumber", skip_serializing_if = "Option::is_none")]
    pub step_number: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_danger: Option<bool>,
    #[serde(default, alias = "actionType", skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>,
    #[serde(default, alias = "actionName", skip_serializing_if = "Option::is_none")]
    pub action_name: Option<String>,
    #[serde(
        default,
        alias = "actionParams",
        skip_serializing_if = "Option::is_none"
    )]
    pub action_params: Option<serde_json::Value>,
}

impl AiPlanStep {
    pub(crate) fn new(
        step: u32,
        title: impl Into<String>,
        command: impl Into<String>,
        description: impl Into<String>,
        is_dangerous: bool,
    ) -> Self {
        let cmd = command.into();
        let is_sudo = cmd.contains("sudo");
        Self {
            step,
            id: Some(format!("step-{step}")),
            title: title.into(),
            command: cmd,
            description: description.into(),
            is_dangerous,
            is_sudo: Some(is_sudo),
            step_number: Some(step),
            is_danger: Some(is_dangerous),
            action_type: Some("shell".to_string()),
            action_name: None,
            action_params: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiExecutionPlan {
    pub id: String,
    pub goal: String,
    pub summary: String,
    #[serde(default, alias = "hostId")]
    pub host_id: Option<String>,
    #[serde(default, alias = "hostLabel")]
    pub host_label: Option<String>,
    pub requirements: Vec<String>,
    pub steps: Vec<AiPlanStep>,
    #[serde(alias = "createdAt")]
    pub created_at: i64,
    #[serde(
        default,
        alias = "estimatedTime",
        skip_serializing_if = "Option::is_none"
    )]
    pub estimated_time: Option<String>,
    #[serde(default = "default_plan_source")]
    pub source: String,
}

fn default_plan_source() -> String {
    "builtin".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiExecutionResult {
    pub step: u32,
    pub command: String,
    pub success: bool,
    #[serde(alias = "exitCode")]
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    #[serde(default, alias = "stepNumber", skip_serializing_if = "Option::is_none")]
    pub step_number: Option<u32>,
    #[serde(default, alias = "durationMs", skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiSettings {
    pub provider: String,
    #[serde(alias = "baseUrl")]
    pub base_url: String,
    #[serde(alias = "apiKey")]
    pub api_key: String,
    pub model: String,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            provider: "custom".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4o".to_string(),
        }
    }
}

pub(crate) fn get_ai_settings_in(conn: &Connection) -> Result<AiSettings, CatermError> {
    let mut stmt = conn
        .prepare("SELECT provider, api_key, base_url, model FROM ai_settings WHERE id = 'default'")
        .map_err(|e| CatermError::Db(DbError::Generic(format!("Failed to prepare query: {e}"))))?;

    let res = stmt.query_row([], |row| {
        Ok(AiSettings {
            provider: row.get(0)?,
            api_key: row.get(1)?,
            base_url: row.get(2)?,
            model: row.get(3)?,
        })
    });

    match res {
        Ok(settings) => Ok(settings),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AiSettings::default()),
        Err(e) => Err(CatermError::Db(DbError::Generic(format!(
            "Failed to load AI settings: {e}"
        )))),
    }
}

pub(crate) fn save_ai_settings_in(
    conn: &Connection,
    settings: AiSettings,
) -> Result<AiSettings, CatermError> {
    conn.execute(
        "INSERT OR REPLACE INTO ai_settings (id, provider, api_key, base_url, model)
         VALUES ('default', ?1, ?2, ?3, ?4);",
        params![
            settings.provider,
            settings.api_key,
            settings.base_url,
            settings.model
        ],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(format!("Failed to save AI settings: {e}"))))?;

    Ok(settings)
}

pub fn get_ai_settings() -> Result<AiSettings, CatermError> {
    let conn = crate::db::open()?;
    get_ai_settings_in(&conn)
}

pub fn save_ai_settings(settings: AiSettings) -> Result<AiSettings, CatermError> {
    let conn = crate::db::open()?;
    save_ai_settings_in(&conn, settings)
}

fn resolve_host_label(target_host: Option<&str>) -> Option<String> {
    let host_id = target_host?;
    let conn = crate::db::open().ok()?;
    let mut stmt = conn.prepare("SELECT label FROM hosts WHERE id = ?1").ok()?;
    stmt.query_row(params![host_id], |row| row.get::<_, String>(0))
        .ok()
        .or_else(|| Some(host_id.to_string()))
}

fn scrub_payload_messages(payload: &mut serde_json::Value) {
    let scrubber = PromptScrubber::new();
    if let Some(messages) = payload.get_mut("messages").and_then(|m| m.as_array_mut()) {
        for msg in messages {
            if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                let scrubbed = scrubber.scrub(content);
                if let Some(c_mut) = msg.get_mut("content") {
                    *c_mut = serde_json::Value::String(scrubbed);
                }
            }
        }
    }
}

fn post_chat_completion(
    settings: &AiSettings,
    mut payload: serde_json::Value,
    timeout: std::time::Duration,
) -> Result<String, CatermError> {
    scrub_payload_messages(&mut payload);
    let base_url = settings.base_url.trim();
    if base_url.is_empty() {
        return Err(CatermError::Ai(AiError::Generic(
            "Base URL AI belum diisi — buka Settings > AI Assistant.".to_string(),
        )));
    }
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .build();
    let agent: ureq::Agent = config.into();

    let mut request = agent.post(&url).header("Content-Type", "application/json");
    let api_key = settings.api_key.trim();
    if !api_key.is_empty() {
        request = request.header("Authorization", &format!("Bearer {api_key}"));
    }

    let mut response = request.send_json(&payload).map_err(|e| {
        CatermError::Ai(AiError::Generic(format!("Failed to connect to {url}: {e}")))
    })?;

    let body = response.body_mut().read_to_string().map_err(|e| {
        CatermError::Ai(AiError::Generic(format!("Failed to read AI response: {e}")))
    })?;

    extract_message_content(&body).ok_or_else(|| {
        CatermError::Ai(AiError::Generic(format!(
            "AI response does not match OpenAI chat/completions format: {}",
            body.chars().take(300).collect::<String>()
        )))
    })
}

fn extract_message_content(body: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
        return value
            .get("choices")?
            .get(0)?
            .get("message")?
            .get("content")?
            .as_str()
            .map(str::to_string);
    }
    extract_streamed_content(body)
}

fn extract_streamed_content(body: &str) -> Option<String> {
    let mut out = String::new();

    for line in body.lines() {
        let Some(payload) = line.trim_start().strip_prefix("data:") else {
            continue;
        };
        let payload = payload.trim();
        if payload.is_empty() || payload == "[DONE]" {
            continue;
        }
        let Ok(chunk) = serde_json::from_str::<serde_json::Value>(payload) else {
            continue;
        };
        let Some(choice) = chunk.get("choices").and_then(|c| c.get(0)) else {
            continue;
        };
        let text = choice
            .get("delta")
            .and_then(|d| d.get("content"))
            .or_else(|| choice.get("message").and_then(|m| m.get("content")))
            .and_then(|c| c.as_str());
        if let Some(text) = text {
            out.push_str(text);
        }
    }

    if out.is_empty() { None } else { Some(out) }
}

fn strip_code_fence(content: &str) -> &str {
    let trimmed = content.trim();
    if let Some(stripped) = trimmed.strip_prefix("```json") {
        stripped.strip_suffix("```").unwrap_or(stripped).trim()
    } else if let Some(stripped) = trimmed.strip_prefix("```") {
        stripped.strip_suffix("```").unwrap_or(stripped).trim()
    } else {
        trimmed
    }
}

const PLAN_SYSTEM_PROMPT: &str = "You are a Linux DevOps and Sysadmin AI assistant. Generate a structured execution plan for the requested goal. Respond ONLY with valid JSON matching this schema:
{
  \"summary\": \"Brief summary of the plan\",
  \"requirements\": [\"Requirement 1\", \"Requirement 2\"],
  \"steps\": [
    {
      \"step\": 1,
      \"title\": \"Step title\",
      \"command\": \"bash command\",
      \"description\": \"Step description\",
      \"is_dangerous\": false
    }
  ]
}";

fn plan_payload(model: Option<&str>, goal: &str) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "messages": [
            { "role": "system", "content": PLAN_SYSTEM_PROMPT },
            { "role": "user", "content": goal }
        ],
        "temperature": 0.2,
        "stream": false
    });
    if let Some(model) = model
        && let Some(obj) = payload.as_object_mut()
    {
        obj.insert(
            "model".to_string(),
            serde_json::Value::String(model.to_string()),
        );
    }
    payload
}

fn call_llm_if_available(
    settings: &AiSettings,
    goal: &str,
    target_host: Option<&str>,
    host_label: Option<String>,
) -> Option<AiExecutionPlan> {
    let payload = plan_payload(Some(&settings.model), goal);
    let content =
        post_chat_completion(settings, payload, std::time::Duration::from_secs(60)).ok()?;
    parse_ai_plan_content(&content, goal, target_host, host_label)
}

fn call_llm_hosted(
    goal: &str,
    target_host: Option<&str>,
    host_label: Option<String>,
) -> Result<AiExecutionPlan, CatermError> {
    let payload = plan_payload(None, goal);
    let body = crate::pro::ai_chat_completion(&payload)?;
    let content = extract_message_content(&body.to_string()).ok_or_else(|| {
        CatermError::Ai(AiError::Generic(format!(
            "AI response does not match OpenAI chat/completions format: {}",
            body.to_string().chars().take(300).collect::<String>()
        )))
    })?;
    parse_ai_plan_content(&content, goal, target_host, host_label).ok_or_else(|| {
        CatermError::Ai(AiError::Generic(
            "Model tidak mengembalikan rencana yang bisa dipakai.".to_string(),
        ))
    })
}

fn parse_plan_steps(parsed: &serde_json::Value) -> Vec<AiPlanStep> {
    let Some(steps_arr) = parsed.get("steps").and_then(|s| s.as_array()) else {
        return Vec::new();
    };

    let mut steps = Vec::new();
    for (idx, item) in steps_arr.iter().enumerate() {
        let step_num = item
            .get("step")
            .and_then(|s| s.as_u64())
            .unwrap_or((idx + 1) as u64) as u32;
        let title = item
            .get("title")
            .and_then(|s| s.as_str())
            .unwrap_or("Execute step")
            .to_string();
        let command = item
            .get("command")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        if command.trim().is_empty() {
            continue;
        }
        let description = item
            .get("description")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let is_dangerous = item
            .get("is_dangerous")
            .and_then(|s| s.as_bool())
            .unwrap_or(false);

        let action_type = item
            .get("actionType")
            .or_else(|| item.get("action_type"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        let action_name = item
            .get("actionName")
            .or_else(|| item.get("action_name"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        let action_params = item
            .get("actionParams")
            .or_else(|| item.get("action_params"))
            .cloned();

        let mut plan_step = AiPlanStep::new(step_num, title, command, description, is_dangerous);
        if action_type.is_some() {
            plan_step.action_type = action_type;
        }
        plan_step.action_name = action_name;
        plan_step.action_params = action_params;

        steps.push(plan_step);
    }

    steps
}

fn parse_ai_plan_content(
    content: &str,
    goal: &str,
    target_host: Option<&str>,
    host_label: Option<String>,
) -> Option<AiExecutionPlan> {
    let parsed: serde_json::Value = serde_json::from_str(strip_code_fence(content)).ok()?;
    let summary = parsed.get("summary")?.as_str()?.to_string();

    let requirements = parsed
        .get("requirements")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let steps = parse_plan_steps(&parsed);

    if steps.is_empty() {
        return None;
    }

    let now = chrono::Utc::now().timestamp_millis();
    let est = format!("~{} mins", (steps.len() * 2).max(1));

    Some(AiExecutionPlan {
        id: format!("plan-{}", uuid::Uuid::new_v4()),
        goal: goal.to_string(),
        summary,
        host_id: target_host.map(|h| h.to_string()),
        host_label,
        requirements,
        steps,
        source: "llm".to_string(),
        created_at: now,
        estimated_time: Some(est),
    })
}

fn build_template_plan(
    goal: &str,
    target_host: Option<&str>,
    host_label: Option<String>,
) -> AiExecutionPlan {
    let goal_lower = goal.to_lowercase();
    let now = chrono::Utc::now().timestamp_millis();

    if goal_lower.contains("laravel")
        || goal_lower.contains("php")
        || goal_lower.contains("composer")
        || (goal_lower.contains("ubuntu")
            && (goal_lower.contains("web") || goal_lower.contains("stack")))
    {
        let requirements = vec![
            "PHP 8.3 CLI & FPM".to_string(),
            "Extensions: mbstring, xml, curl, zip, mysql, bcmath, intl, gd, sqlite3".to_string(),
            "Composer v2".to_string(),
            "MySQL server".to_string(),
            "Node.js 20 & NPM".to_string(),
            "Git".to_string(),
        ];

        let steps = vec![
            AiPlanStep::new(
                1,
                "Update system packages",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get update -y && sudo apt-get install -y curl git unzip software-properties-common ca-certificates",
                "Update apt cache and install base prerequisite packages",
                false,
            ),
            AiPlanStep::new(
                2,
                "Add ondrej/php PPA",
                "sudo add-apt-repository -y ppa:ondrej/php && sudo apt-get update -y",
                "Add Ondřej Surý PPA repository for PHP 8.3",
                false,
            ),
            AiPlanStep::new(
                3,
                "Install PHP 8.3 and extensions",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get install -y php8.3 php8.3-cli php8.3-fpm php8.3-mbstring php8.3-xml php8.3-curl php8.3-zip php8.3-mysql php8.3-bcmath php8.3-intl php8.3-gd php8.3-sqlite3",
                "Install PHP 8.3 core, FPM, and required extensions (mbstring, xml, curl, zip, mysql, bcmath, intl, gd, sqlite3)",
                false,
            ),
            AiPlanStep::new(
                4,
                "Install Composer v2",
                "curl -sS https://getcomposer.org/installer | php && sudo mv composer.phar /usr/local/bin/composer && sudo chmod +x /usr/local/bin/composer",
                "Download and install Composer v2 package manager globally",
                false,
            ),
            AiPlanStep::new(
                5,
                "Install MySQL Server",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get install -y mysql-server && sudo systemctl enable --now mysql",
                "Install MySQL server package and start the daemon service",
                false,
            ),
            AiPlanStep::new(
                6,
                "Install Node.js 20 & NPM",
                "curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - && export DEBIAN_FRONTEND=noninteractive && sudo apt-get install -y nodejs",
                "Set up NodeSource repository and install Node.js 20 LTS and NPM for frontend asset compilation",
                false,
            ),
            AiPlanStep::new(
                7,
                "Verify versions",
                "php -v && composer --version && mysql --version && node -v && npm -v",
                "Verify installed versions of PHP 8.3, Composer, MySQL, and Node.js",
                false,
            ),
        ];

        AiExecutionPlan {
            id: format!("plan-{}", uuid::Uuid::new_v4()),
            goal: goal.to_string(),
            summary: "Set up production-ready Laravel environment with PHP 8.3, Composer v2, MySQL, and Node.js 20 on Ubuntu.".to_string(),
            host_id: target_host.map(|h| h.to_string()),
            host_label,
            requirements,
            steps,
            source: default_plan_source(),
            created_at: now,
            estimated_time: Some("~12 mins".to_string()),
        }
    } else {
        let requirements = vec![
            "Bash shell environment".to_string(),
            "Sudo/root administrative privileges".to_string(),
            "System package manager (apt)".to_string(),
        ];

        let steps = vec![
            AiPlanStep::new(
                1,
                "Inspect system information",
                "uname -a && cat /etc/os-release",
                "Check operating system release and kernel details",
                false,
            ),
            AiPlanStep::new(
                2,
                "Update package index",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get update -y",
                "Update local repository package lists",
                false,
            ),
            AiPlanStep::new(
                3,
                "Check disk and memory resources",
                "df -h && free -m",
                "Check storage partitions and RAM utilization before proceeding",
                false,
            ),
        ];

        AiExecutionPlan {
            id: format!("plan-{}", uuid::Uuid::new_v4()),
            goal: goal.to_string(),
            summary: format!("Execution plan for: {goal}"),
            host_id: target_host.map(|h| h.to_string()),
            host_label,
            requirements,
            steps,
            source: default_plan_source(),
            created_at: now,
            estimated_time: Some("~3 mins".to_string()),
        }
    }
}

pub fn generate_plan(
    goal: &str,
    target_host: Option<&str>,
    hosted: bool,
) -> Result<AiExecutionPlan, CatermError> {
    if goal.trim().is_empty() {
        return Err(CatermError::Ai(AiError::Generic(
            "Goal cannot be empty".to_string(),
        )));
    }

    let host_label = resolve_host_label(target_host);

    if hosted {
        return call_llm_hosted(goal, target_host, host_label);
    }

    if let Ok(settings) = get_ai_settings()
        && let Some(plan) = call_llm_if_available(&settings, goal, target_host, host_label.clone())
    {
        return Ok(plan);
    }

    Ok(build_template_plan(goal, target_host, host_label))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiChatReply {
    pub reply: String,
    pub ready: bool,
    pub steps: Vec<AiPlanStep>,
}

const CHAT_SYSTEM_PROMPT: &str = "You are Hana AI 🌸, CATerm's calm, sharp, and reliable DevOps Co-Pilot & In-App Assistant embedded in the SSH terminal client.

Your core traits:
- Calm, composed, and straight-to-the-point under pressure.
- Action-oriented: you generate concrete Linux commands, snippet collections, configuration solutions, AND native in-app CATerm actions.
- Security-first & Zero-Knowledge aware: you warn before destructive operations (rm -rf, drop table, kill -9) and mask credentials.

Respond with ONLY a valid JSON object matching this schema:
{\"reply\": \"what you say to the user\", \"ready\": true_or_false, \"steps\": [{\"step\": 1, \"title\": \"...\", \"command\": \"...\", \"description\": \"...\", \"is_dangerous\": false}]}";

pub fn chat(
    messages: Vec<AiChatMessage>,
    host_label: Option<&str>,
    hosted: bool,
) -> Result<AiChatReply, CatermError> {
    if messages.is_empty() {
        return Err(CatermError::Validation(ValidationError::Generic(
            "Percakapan kosong".to_string(),
        )));
    }

    let mut system = CHAT_SYSTEM_PROMPT.to_string();
    if let Some(label) = host_label.filter(|l| !l.trim().is_empty()) {
        system.push_str(&format!("\n\nThe commands will run on the host the user calls \"{label}\"."));
    }

    let mut payload_messages = vec![serde_json::json!({ "role": "system", "content": system })];
    for message in &messages {
        let role = match message.role.as_str() {
            "assistant" => "assistant",
            "system" => "system",
            _ => "user",
        };
        payload_messages
            .push(serde_json::json!({ "role": role, "content": message.content.clone() }));
    }

    let content = if hosted {
        let payload = serde_json::json!({
            "messages": payload_messages,
            "temperature": 0.3,
            "stream": false
        });
        let body = crate::pro::ai_chat_completion(&payload)?;
        extract_message_content(&body.to_string()).ok_or_else(|| {
            CatermError::Ai(AiError::Generic(format!(
                "AI response does not match OpenAI chat/completions format: {}",
                body.to_string().chars().take(300).collect::<String>()
            )))
        })?
    } else {
        let settings = get_ai_settings()?;
        let payload = serde_json::json!({
            "model": &settings.model,
            "messages": payload_messages,
            "temperature": 0.3,
            "stream": false
        });
        post_chat_completion(&settings, payload, std::time::Duration::from_secs(120))?
    };

    Ok(parse_chat_reply(&content))
}

fn parse_chat_reply(content: &str) -> AiChatReply {
    let stripped = strip_code_fence(content);

    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(stripped) else {
        return AiChatReply {
            reply: content.trim().to_string(),
            ready: false,
            steps: Vec::new(),
        };
    };

    let reply = parsed
        .get("reply")
        .and_then(|r| r.as_str())
        .unwrap_or_else(|| content.trim())
        .to_string();

    let ready = parsed
        .get("ready")
        .and_then(|r| r.as_bool())
        .unwrap_or(false);
    let steps = if ready {
        parse_plan_steps(&parsed)
    } else {
        Vec::new()
    };
    let ready = ready && !steps.is_empty();

    AiChatReply {
        reply,
        ready,
        steps,
    }
}

pub fn execute_plan_step(host_id: &str, command: &str) -> Result<AiExecutionResult, CatermError> {
    if host_id.trim().is_empty() {
        return Err(CatermError::Validation(ValidationError::Generic(
            "host_id cannot be empty".to_string(),
        )));
    }
    if command.trim().is_empty() {
        return Err(CatermError::Validation(ValidationError::Generic(
            "command cannot be empty".to_string(),
        )));
    }

    if host_id == "local" || host_id == "__local__" {
        #[cfg(windows)]
        let mut cmd = std::process::Command::new("powershell.exe");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.args(["-NoLogo", "-NonInteractive", "-Command", command]);
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        #[cfg(not(windows))]
        let mut cmd = std::process::Command::new("sh");
        #[cfg(not(windows))]
        cmd.args(["-c", command]);

        let output = cmd.output().map_err(|e| {
            CatermError::Ai(AiError::Generic(format!(
                "Failed to execute local command: {e}"
            )))
        })?;

        let exit_code = output.status.code().unwrap_or(0);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();
        let summary = if success { "SUCCESS" } else { "FAILED" };

        let _ = crate::audit::log_event(
            "AI_AUTOMATION",
            Some(host_id),
            &format!("[AI-EXEC-LOCAL] Command: {command} Exit: {exit_code} Result: {summary}"),
        );

        return Ok(AiExecutionResult {
            step: 1,
            command: command.to_string(),
            success,
            exit_code,
            stdout,
            stderr,
            step_number: Some(1),
            duration_ms: None,
        });
    }

    let (exit_code, stdout_buf, stderr_buf) = crate::ssh::with_exec_session(host_id, |sess| {
        let mut channel = sess.channel_session().map_err(|e| {
            CatermError::Ssh(SshError::Generic(format!(
                "Failed to open SSH channel: {e}"
            )))
        })?;

        channel.exec(command).map_err(|e| {
            CatermError::Ssh(SshError::Generic(format!("Failed to execute command: {e}")))
        })?;

        use std::io::Read;
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();

        let _ = channel.read_to_end(&mut stdout_buf);
        let _ = channel.stderr().read_to_end(&mut stderr_buf);
        let _ = channel.wait_close();

        Ok((channel.exit_status().unwrap_or(0), stdout_buf, stderr_buf))
    })?;

    let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
    let stderr = String::from_utf8_lossy(&stderr_buf).to_string();
    let success = exit_code == 0;
    let summary = if success { "SUCCESS" } else { "FAILED" };

    let _ = crate::audit::log_event(
        "AI_AUTOMATION",
        Some(host_id),
        &format!("[AI-EXEC] Command: {command} | Exit: {exit_code} | Result: {summary}"),
    );

    Ok(AiExecutionResult {
        step: 1,
        command: command.to_string(),
        success,
        exit_code,
        stdout,
        stderr,
        step_number: Some(1),
        duration_ms: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TempDb(PathBuf);
    impl TempDb {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            Self(std::env::temp_dir().join(format!("caterm_ai_test_{label}_{nanos:x}")))
        }
    }
    impl Drop for TempDb {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn test_context_builder_lifecycle() {
        let t = TempDb::new("context_lifecycle");
        let conn = crate::db::open_encrypted(&t.0, "test-ai-key").expect("db");

        // Save a habit
        let habit = HabitFact {
            id: uuid::Uuid::new_v4().to_string(),
            category: "cli_preference".to_string(),
            key_tag: "docker_runtime".to_string(),
            fact_content: "Prefers podman over docker".to_string(),
            source_context: Some("dev terminal".to_string()),
            confidence_score: 1.0,
            occurrence_count: 5,
            is_pinned: true,
            is_active: true,
            created_at: 0,
            last_accessed_at: 0,
        };
        db::save_habit(&conn, &habit).expect("save habit");

        let builder = ContextBuilder::new(&conn);
        let term_ctx = TerminalContext {
            tail_lines: vec![
                "curl -H \"Authorization: Bearer secret123\" http://localhost".to_string(),
                "Exit: 1".to_string(),
            ],
            exit_code: Some(1),
            cwd: Some("/var/www".to_string()),
            user: Some("deploy".to_string()),
            host_label: Some("srv-prod-01".to_string()),
            active_host_id: None,
            shell: Some("zsh".to_string()),
            os_info: Some("Ubuntu 24.04".to_string()),
        };

        let assembled = builder
            .build("how do I restart podman?", None, Some(&term_ctx), 2048)
            .expect("build context");

        assert!(assembled.system_prompt.contains("[SYSTEM CONTEXT: PERSONA]"));
        assert!(assembled.system_prompt.contains("[ENVIRONMENT CONSTRAINTS]"));
        assert!(assembled.system_prompt.contains("Prefers podman over docker"));
        assert!(assembled.user_prompt.contains("[REDACTED_BEARER_TOKEN]"));
        assert!(!assembled.user_prompt.contains("secret123"));
    }

    #[test]
    fn test_habit_learner() {
        let t = TempDb::new("learner");
        let conn = crate::db::open_encrypted(&t.0, "test-ai-key").expect("db");

        let learned = HabitLearner::observe_command(&conn, "podman-compose up -d", Some(0), Some("server-1"))
            .expect("observe");
        assert!(learned.is_some());
        let h = learned.expect("some habit");
        assert_eq!(h.key_tag, "container_runtime");
        assert!(h.fact_content.contains("podman-compose"));
    }

    #[test]
    fn test_task_routing_matrix_query() {
        let t = TempDb::new("matrix");
        let conn = crate::db::open_encrypted(&t.0, "test-ai-key").expect("db");

        let matrix = db::get_routing_matrix(&conn).expect("matrix");
        assert!(!matrix.is_empty(), "Default routing matrix should be seeded");

        let err_diag = db::get_route_rule(&conn, TaskType::ErrorDiagnostic)
            .expect("get rule")
            .expect("rule exists");
        assert_eq!(err_diag.task_type, TaskType::ErrorDiagnostic);
        assert_eq!(err_diag.primary_provider_id, "byo-anthropic");
    }
}

