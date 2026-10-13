//! Shared HTTP client + OpenAI error-response parsing for the MOM and
//! AI Notes generation modules (Chat Completions API, structured JSON
//! output). The transcription module keeps its own separate, already
//! tested copy of similar logic rather than being refactored to share
//! this - deliberately avoiding touching proven working code just to
//! remove a small amount of duplication.

use serde::Deserialize;

pub fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    message: String,
}

fn extract_error_message(status: reqwest::StatusCode, body_text: &str) -> String {
    if let Ok(err_body) = serde_json::from_str::<ApiErrorBody>(body_text) {
        format!("OpenAI API error ({status}): {}", err_body.error.message)
    } else {
        format!("OpenAI API error ({status}): {body_text}")
    }
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

/// Calls the Chat Completions API with strict JSON-schema structured
/// output. Returns the raw JSON string the model produced (guaranteed
/// to match `schema` by OpenAI's strict mode) - callers parse it into
/// their own specific struct.
pub async fn call_chat_json(
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_content: &str,
    schema_name: &str,
    schema: serde_json::Value,
) -> Result<String, String> {
    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_content}
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": schema_name,
                "strict": true,
                "schema": schema
            }
        }
    });

    let body_string =
        serde_json::to_string(&body).map_err(|e| format!("serialize request body: {e}"))?;

    let http = client()?;
    let response = http
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .body(body_string)
        .send()
        .await
        .map_err(|e| format!("network error calling OpenAI: {e}"))?;

    let status = response.status();
    let body_text = response
        .text()
        .await
        .map_err(|e| format!("read OpenAI response body: {e}"))?;

    if !status.is_success() {
        return Err(extract_error_message(status, &body_text));
    }

    let parsed: ChatCompletionResponse = serde_json::from_str(&body_text).map_err(|e| {
        format!(
            "could not parse OpenAI's response - raw response: {body_text} | parse error: {e}"
        )
    })?;

    parsed
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .ok_or_else(|| format!("OpenAI returned no content - raw response: {body_text}"))
}