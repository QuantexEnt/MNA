//! AI Notes generation - real mode (OpenAI Chat Completions, strict
//! JSON-schema structured output). A shorter, informal summary of key
//! points, distinct from the formal structured MOM. No mock mode,
//! matching the "real, not mock" decision used throughout.

use crate::generation_input::{self, GenerationInput};
use crate::openai_http;
use crate::session;
use crate::settings;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiNotes {
    pub meeting_id: String,
    pub bullets: Vec<String>,
    pub model: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct AiNotesModelOutput {
    bullets: Vec<String>,
}

const AI_NOTES_SYSTEM_PROMPT: &str = "You are writing a short, informal set of AI-generated notes summarizing the key points of a meeting, based on its transcript and/or the user's personal notes.\n\nRules:\n1. Use only information actually present in the supplied material. Do not invent facts, people, or outcomes.\n2. Write each point as a short, casual bullet - like what an attentive colleague might jot down live, not a formal document.\n3. Do not infer decisions or commitments that were not actually and clearly made.\n4. Do not perform psychological, emotional, or sentiment analysis of any participant.\n5. Keep it brief: aim for roughly 5 to 10 bullets covering the most relevant points, not an exhaustive recap of the whole conversation.";

fn ai_notes_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "bullets": {"type": "array", "items": {"type": "string"}}
        },
        "required": ["bullets"]
    })
}

fn ai_notes_to_markdown(notes: &AiNotes) -> String {
    let mut md = String::new();
    md.push_str("# AI Notes\n\n");
    for b in &notes.bullets {
        md.push_str(&format!("- {b}\n"));
    }
    md
}

fn write_ai_notes_set(
    root: &PathBuf,
    notes: &AiNotes,
    also_write_generated: bool,
) -> Result<(), String> {
    let ai_notes_dir = root.join("ai_notes");
    fs::create_dir_all(&ai_notes_dir).map_err(|e| format!("create ai_notes dir: {e}"))?;

    let json =
        serde_json::to_string_pretty(notes).map_err(|e| format!("serialize ai notes: {e}"))?;
    let md = ai_notes_to_markdown(notes);

    if also_write_generated {
        fs::write(ai_notes_dir.join("ai_notes_generated.json"), &json)
            .map_err(|e| format!("write ai_notes_generated.json: {e}"))?;
        fs::write(ai_notes_dir.join("ai_notes_generated.md"), &md)
            .map_err(|e| format!("write ai_notes_generated.md: {e}"))?;
    }
    fs::write(ai_notes_dir.join("ai_notes_final.json"), &json)
        .map_err(|e| format!("write ai_notes_final.json: {e}"))?;
    fs::write(ai_notes_dir.join("ai_notes_final.md"), &md)
        .map_err(|e| format!("write ai_notes_final.md: {e}"))?;
    Ok(())
}

async fn call_ai_notes_model(
    app_settings: &settings::AppSettings,
    input: &GenerationInput,
    meeting_id: &str,
) -> Result<AiNotes, String> {
    let user_content = generation_input::build_user_content(input);
    let raw_json = openai_http::call_chat_json(
        &app_settings.openai_api_key,
        &app_settings.mom_model,
        AI_NOTES_SYSTEM_PROMPT,
        &user_content,
        "ai_notes",
        ai_notes_json_schema(),
    )
    .await?;

    let parsed: AiNotesModelOutput = serde_json::from_str(&raw_json).map_err(|e| {
        format!(
            "could not parse the AI's Notes output - raw output: {raw_json} | parse error: {e}"
        )
    })?;

    Ok(AiNotes {
        meeting_id: meeting_id.to_string(),
        bullets: parsed.bullets,
        model: app_settings.mom_model.clone(),
        created_at: chrono::Local::now().to_rfc3339(),
    })
}

#[tauri::command]
pub async fn generate_ai_notes(meeting_id: String) -> Result<AiNotes, String> {
    let root = session::find_meeting_root(&meeting_id)?;
    let input = generation_input::build_generation_input(&meeting_id)?;
    if input.transcript_text.is_none() && input.manual_notes.is_none() {
        return Err(
            "No transcript or notes are available for this meeting yet - nothing to generate AI Notes from."
                .to_string(),
        );
    }

    let app_settings = settings::load_settings()?;
    if app_settings.openai_api_key.trim().is_empty() {
        return Err(
            "no OpenAI API key is configured - add one in Settings before generating AI Notes"
                .to_string(),
        );
    }

    session::update_ai_notes_status(&root, "GENERATING")?;

    let result = call_ai_notes_model(&app_settings, &input, &meeting_id).await;

    match result {
        Ok(notes) => {
            write_ai_notes_set(&root, &notes, true)?;
            session::update_ai_notes_status(&root, "GENERATED")?;
            Ok(notes)
        }
        Err(e) => {
            let _ = session::update_ai_notes_status(&root, "FAILED");
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn regenerate_ai_notes(meeting_id: String) -> Result<AiNotes, String> {
    generate_ai_notes(meeting_id).await
}

#[tauri::command]
pub fn get_ai_notes(meeting_id: String) -> Result<AiNotes, String> {
    let root = session::find_meeting_root(&meeting_id)?;
    let content = fs::read_to_string(root.join("ai_notes").join("ai_notes_final.json"))
        .map_err(|e| format!("read ai_notes_final.json: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("parse ai_notes_final.json: {e}"))
}

#[tauri::command]
pub fn get_ai_notes_markdown(meeting_id: String) -> Result<String, String> {
    let root = session::find_meeting_root(&meeting_id)?;
    fs::read_to_string(root.join("ai_notes").join("ai_notes_final.md"))
        .map_err(|e| format!("read ai_notes_final.md: {e}"))
}

#[tauri::command]
pub fn save_ai_notes(meeting_id: String, notes: AiNotes) -> Result<(), String> {
    let root = session::find_meeting_root(&meeting_id)?;
    write_ai_notes_set(&root, &notes, false)?;
    session::update_ai_notes_status(&root, "EDITED")?;
    Ok(())
}