use serde::{Deserialize, Serialize};

/// A single fact extracted by the LLM from conversation messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedMemory {
    pub content: String,
    #[serde(default = "default_importance")]
    pub importance: String,
    #[serde(default)]
    pub suggested_ttl_seconds: Option<i64>,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_importance() -> String {
    "medium".into()
}

/// Message in a conversation (role + content).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

const EXTRACTION_SYSTEM_PROMPT: &str = r#"You are a memory extraction engine. Given a conversation, extract discrete, atomic facts worth remembering long-term.

For each fact, provide:
- content: the fact as a clear, standalone statement
- importance: "low", "medium", or "high"
- suggested_ttl_seconds: null for permanent, or seconds for ephemeral
- tags: relevant tags as array of strings

Return a JSON array. Only extract genuinely useful facts. Skip pleasantries, filler, and obvious context. If there are no facts worth remembering, return an empty array [].

Example output:
[{"content":"User prefers dark mode","importance":"medium","suggested_ttl_seconds":null,"tags":["preference","ui"]}]"#;

/// Call OpenAI chat completions to extract memories from conversation messages.
pub async fn extract_memories(
    http: &reqwest::Client,
    messages: &[ChatMessage],
    model: &str,
    api_key: &str,
) -> anyhow::Result<Vec<ExtractedMemory>> {
    // Build the conversation for the LLM
    let mut llm_messages = vec![serde_json::json!({
        "role": "system",
        "content": EXTRACTION_SYSTEM_PROMPT
    })];

    // Add the user's conversation as a single user message
    let conversation_text: String = messages
        .iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    llm_messages.push(serde_json::json!({
        "role": "user",
        "content": format!("Extract memories from this conversation:\n\n{}", conversation_text)
    }));

    let body = serde_json::json!({
        "model": model,
        "messages": llm_messages,
        "temperature": 0.1,
        "response_format": { "type": "json_object" }
    });

    let resp = http
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("OpenAI chat API error {}: {}", status, text);
    }

    let json: serde_json::Value = resp.json().await?;
    let content_str = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("no content in LLM response"))?;

    // Parse the response - it might be {"memories": [...]} or just [...]
    let parsed: serde_json::Value = serde_json::from_str(content_str)
        .map_err(|e| anyhow::anyhow!("failed to parse LLM JSON: {} — raw: {}", e, content_str))?;

    let memories: Vec<ExtractedMemory> = if let Some(arr) = parsed.as_array() {
        serde_json::from_value(serde_json::Value::Array(arr.clone()))?
    } else if let Some(arr) = parsed.get("memories").and_then(|v| v.as_array()) {
        serde_json::from_value(serde_json::Value::Array(arr.clone()))?
    } else if let Some(arr) = parsed.get("facts").and_then(|v| v.as_array()) {
        serde_json::from_value(serde_json::Value::Array(arr.clone()))?
    } else if parsed.is_object() && parsed.get("content").is_some() {
        // Single object returned instead of array
        vec![serde_json::from_value(parsed)?]
    } else {
        anyhow::bail!("LLM returned unexpected JSON structure: {}", content_str);
    };

    Ok(memories)
}

/// Generic single-prompt LLM call. Returns the assistant's text response.
pub async fn call_llm(api_key: &str, model: &str, prompt: &str) -> anyhow::Result<String> {
    let http = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.3,
    });

    let resp = http
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("OpenAI API error {}: {}", status, text);
    }

    let json: serde_json::Value = resp.json().await?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("no content in LLM response"))?;

    Ok(content.to_string())
}

/// Validate a new memory against existing memories for contradictions.
/// Returns (pass, conflicts) where pass=true means no contradictions found.
pub async fn validate_memory(
    api_key: &str,
    model: &str,
    new_content: &str,
    existing: &[String],
) -> anyhow::Result<(bool, Vec<String>)> {
    if existing.is_empty() {
        return Ok((true, vec![]));
    }

    let existing_list = existing
        .iter()
        .enumerate()
        .map(|(i, m)| format!("{}. {}", i + 1, m))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "Does the following new memory contradict any of these existing facts?\n\n\
         New memory: \"{}\"\n\n\
         Existing facts:\n{}\n\n\
         Reply with JSON {{\"pass\": true/false, \"conflicts\": [\"explanation...\"]}}. \
         If there are no contradictions, pass should be true and conflicts should be empty.",
        new_content, existing_list
    );

    let response = call_llm(api_key, model, &prompt).await?;

    // Try to parse JSON from the response (handle markdown code blocks)
    let json_str = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(val) => {
            let pass = val.get("pass").and_then(|v| v.as_bool()).unwrap_or(true);
            let conflicts = val
                .get("conflicts")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            Ok((pass, conflicts))
        }
        Err(_) => {
            // If we can't parse, assume pass
            tracing::warn!("Could not parse validation response as JSON: {}", response);
            Ok((true, vec![]))
        }
    }
}
