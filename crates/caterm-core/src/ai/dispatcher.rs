//! Multi-Provider AI Dispatcher with Automatic Fallback Chain.
//! Supports Anthropic, OpenAI-compatible, Ollama, and CATerm Pro Hosted AI.

use super::context_builder::ContextBuilder;
use super::db::{get_provider, get_route_rule};
use super::models::{
    AiProviderConfig, DispatchResult, ProviderType, TaskRouteRule, TaskType, TerminalContext,
};
use crate::error::{AiError, CatermError};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::time::Duration;

pub struct Dispatcher<'a> {
    conn: &'a Connection,
}

impl<'a> Dispatcher<'a> {
    /// # Infallible: wraps reference to database connection
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Dispatch a task based on routing matrix with automatic fallback.
    pub fn dispatch(
        &self,
        task_type: TaskType,
        user_query: &str,
        terminal_ctx: Option<&TerminalContext>,
    ) -> Result<DispatchResult, CatermError> {
        // 1. Resolve route rule
        let rule = get_route_rule(self.conn, task_type)?.unwrap_or_else(|| TaskRouteRule {
            task_type,
            primary_provider_id: "byo-anthropic".to_string(),
            primary_model: "claude-3-7-sonnet-20250219".to_string(),
            fallback_provider_id: Some("byo-openai".to_string()),
            fallback_model: Some("gpt-4o".to_string()),
            temperature: 0.2,
            max_tokens: 2048,
            system_persona_id: Some("default-devops".to_string()),
        });

        // 2. Assemble context
        let builder = ContextBuilder::new(self.conn);
        let assembled = builder.build(
            user_query,
            rule.system_persona_id.as_deref(),
            terminal_ctx,
            rule.max_tokens,
        )?;

        // 3. Resolve primary provider
        let primary_provider = get_provider(self.conn, &rule.primary_provider_id)?
            .ok_or_else(|| {
                CatermError::Ai(AiError::Generic(format!(
                    "Primary AI provider '{}' not found in registry",
                    rule.primary_provider_id
                )))
            })?;

        let timeout = Duration::from_secs(15);

        // 4. Try primary provider
        match Self::execute_provider_call(
            &primary_provider,
            &rule.primary_model,
            &assembled.system_prompt,
            &assembled.user_prompt,
            rule.temperature,
            rule.max_tokens,
            timeout,
        ) {
            Ok(content) => Ok(DispatchResult {
                content,
                provider_id: primary_provider.id,
                provider_type: primary_provider.provider_type,
                model: rule.primary_model,
                fell_back: false,
                prompt_tokens: None,
                completion_tokens: None,
            }),
            Err(primary_err) => {
                // If fallback provider configured, attempt fallback
                if let (Some(fb_id), Some(fb_model)) = (
                    &rule.fallback_provider_id,
                    &rule.fallback_model,
                ) {
                    if let Ok(Some(fallback_provider)) = get_provider(self.conn, fb_id) {
                        if fallback_provider.is_active {
                            if let Ok(content) = Self::execute_provider_call(
                                &fallback_provider,
                                fb_model,
                                &assembled.system_prompt,
                                &assembled.user_prompt,
                                rule.temperature,
                                rule.max_tokens,
                                timeout,
                            ) {
                                return Ok(DispatchResult {
                                    content,
                                    provider_id: fallback_provider.id,
                                    provider_type: fallback_provider.provider_type,
                                    model: fb_model.clone(),
                                    fell_back: true,
                                    prompt_tokens: None,
                                    completion_tokens: None,
                                });
                            }
                        }
                    }
                }

                // If fallback failed or absent, return primary error
                Err(primary_err)
            }
        }
    }

    fn execute_provider_call(
        provider: &AiProviderConfig,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: u32,
        timeout: Duration,
    ) -> Result<String, CatermError> {
        match provider.provider_type {
            ProviderType::Anthropic => Self::call_anthropic(
                provider,
                model,
                system_prompt,
                user_prompt,
                temperature,
                max_tokens,
                timeout,
            ),
            ProviderType::OpenaiCompatible => Self::call_openai_compatible(
                provider,
                model,
                system_prompt,
                user_prompt,
                temperature,
                max_tokens,
                timeout,
            ),
            ProviderType::Ollama => Self::call_ollama(
                provider,
                model,
                system_prompt,
                user_prompt,
                temperature,
                max_tokens,
                timeout,
            ),
            ProviderType::CatermHosted => Self::call_caterm_hosted(
                model,
                system_prompt,
                user_prompt,
                temperature,
                max_tokens,
            ),
        }
    }

    fn call_anthropic(
        provider: &AiProviderConfig,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: u32,
        timeout: Duration,
    ) -> Result<String, CatermError> {
        let base_url = provider.base_url.trim().trim_end_matches('/');
        let url = format!("{base_url}/messages");

        let payload = json!({
            "model": model,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "system": system_prompt,
            "messages": [
                { "role": "user", "content": user_prompt }
            ]
        });

        let config = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            .build();
        let agent: ureq::Agent = config.into();

        let mut req = agent
            .post(&url)
            .header("content-type", "application/json")
            .header("anthropic-version", "2023-06-01");

        if let Some(key) = &provider.api_key {
            let k = key.trim();
            if !k.is_empty() {
                req = req.header("x-api-key", k);
            }
        }

        if let Some(headers) = &provider.custom_headers {
            for (k, v) in headers {
                req = req.header(k, v);
            }
        }

        let mut res = req
            .send_json(&payload)
            .map_err(|e| CatermError::Ai(AiError::Generic(format!("Anthropic error: {e}"))))?;

        let body = res
            .body_mut()
            .read_to_string()
            .map_err(|e| CatermError::Ai(AiError::Generic(format!("Read Anthropic response: {e}"))))?;

        let parsed: Value = serde_json::from_str(&body)
            .map_err(|e| CatermError::Ai(AiError::Generic(format!("Parse Anthropic JSON: {e}"))))?;

        if let Some(err_msg) = parsed.get("error").and_then(|e| e.get("message")).and_then(Value::as_str) {
            return Err(CatermError::Ai(AiError::Generic(format!("Anthropic API returned error: {err_msg}"))));
        }

        let text = parsed
            .get("content")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CatermError::Ai(AiError::Generic(format!(
                    "Unexpected Anthropic response shape: {}",
                    body.chars().take(200).collect::<String>()
                )))
            })?;

        Ok(text.to_string())
    }

    fn call_openai_compatible(
        provider: &AiProviderConfig,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: u32,
        timeout: Duration,
    ) -> Result<String, CatermError> {
        let base_url = provider.base_url.trim().trim_end_matches('/');
        let url = format!("{base_url}/chat/completions");

        let payload = json!({
            "model": model,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ]
        });

        let config = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            .build();
        let agent: ureq::Agent = config.into();

        let mut req = agent.post(&url).header("content-type", "application/json");

        if let Some(key) = &provider.api_key {
            let k = key.trim();
            if !k.is_empty() {
                req = req.header("authorization", &format!("Bearer {k}"));
            }
        }

        if let Some(headers) = &provider.custom_headers {
            for (k, v) in headers {
                req = req.header(k, v);
            }
        }

        let mut res = req
            .send_json(&payload)
            .map_err(|e| CatermError::Ai(AiError::Generic(format!("OpenAI error: {e}"))))?;

        let body = res
            .body_mut()
            .read_to_string()
            .map_err(|e| CatermError::Ai(AiError::Generic(format!("Read OpenAI response: {e}"))))?;

        let parsed: Value = serde_json::from_str(&body)
            .map_err(|e| CatermError::Ai(AiError::Generic(format!("Parse OpenAI JSON: {e}"))))?;

        if let Some(err_msg) = parsed.get("error").and_then(|e| e.get("message")).and_then(Value::as_str) {
            return Err(CatermError::Ai(AiError::Generic(format!("OpenAI API returned error: {err_msg}"))));
        }

        let content = parsed
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CatermError::Ai(AiError::Generic(format!(
                    "Unexpected OpenAI response shape: {}",
                    body.chars().take(200).collect::<String>()
                )))
            })?;

        Ok(content.to_string())
    }

    fn call_ollama(
        provider: &AiProviderConfig,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: u32,
        timeout: Duration,
    ) -> Result<String, CatermError> {
        let base_url = provider.base_url.trim().trim_end_matches('/');
        let url = format!("{base_url}/api/chat");

        let payload = json!({
            "model": model,
            "stream": false,
            "options": {
                "temperature": temperature,
                "num_predict": max_tokens
            },
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ]
        });

        let config = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            .build();
        let agent: ureq::Agent = config.into();

        let req = agent.post(&url).header("content-type", "application/json");

        let mut res = req
            .send_json(&payload)
            .map_err(|e| CatermError::Ai(AiError::Generic(format!("Ollama error: {e}"))))?;

        let body = res
            .body_mut()
            .read_to_string()
            .map_err(|e| CatermError::Ai(AiError::Generic(format!("Read Ollama response: {e}"))))?;

        let parsed: Value = serde_json::from_str(&body)
            .map_err(|e| CatermError::Ai(AiError::Generic(format!("Parse Ollama JSON: {e}"))))?;

        let content = parsed
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CatermError::Ai(AiError::Generic(format!(
                    "Unexpected Ollama response shape: {}",
                    body.chars().take(200).collect::<String>()
                )))
            })?;

        Ok(content.to_string())
    }

    fn call_caterm_hosted(
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> Result<String, CatermError> {
        let payload = json!({
            "model": model,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ]
        });

        let res = crate::pro::ai_chat_completion(&payload)?;
        let content = res
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CatermError::Ai(AiError::Generic("CATerm Pro AI returned invalid response".to_string()))
            })?;

        Ok(content.to_string())
    }
}
