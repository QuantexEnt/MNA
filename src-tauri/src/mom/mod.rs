//! MOM (Minutes of Meeting) generation - real mode (OpenAI Chat
//! Completions, strict JSON-schema structured output). No mock mode,
//! matching the "real transcriptions, not mock" decision extended
//! consistently here.
//!
//! The generated draft and the user's edited final version are always
//! kept as separate files (mom_generated.* vs mom_final.*), so
//! regeneration never silently destroys user edits
//! (NOTES_AND_MOM_SPECIFICATION.md §30, LOCAL_STORAGE_SPECIFICATION.md
//! §14-15).
//!
//! Anti-hallucination rules are enforced via the system prompt
//! (Decision 031/032/033/034, NOTES_AND_MOM_SPECIFICATION.md §22-26).
//!
//! Uses the same `mom_model` setting for both MOM and AI Notes (no
//! separate ai_notes_model field) - a deliberate simplification.
//!
//! Context length: the full transcript (plus notes) is sent as one
//! message with no chunking/summarization step. Should comfortably fit
//! current flagship-model context windows even for multi-hour meetings,
//! but this has not been empirically verified against a real long
//! transcript yet.

use crate::generation_input::{self, GenerationInput};
use crate::openai_http;
use crate::session;
use crate::settings;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub action: String,
    pub owner: String,
    pub due_date: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mom {
    pub meeting_id: String,
    pub title: String,
    pub date: String,
    pub participants: Vec<String>,
    pub purpose: String,
    pub discussion_points: Vec<String>,
    pub decisions: Vec<String>,
    pub action_items: Vec<ActionItem>,
    pub open_questions: Vec<String>,
    pub next_steps: Vec<String>,
    pub model: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct MomModelOutput {
    title: String,
    date: String,
    participants: Vec<String>,
    purpose: String,
    discussion_points: Vec<String>,
    decisions: Vec<String>,
    action_items: Vec<ActionItem>,
    open_questions: Vec<String>,
    next_steps: Vec<String>,
}

const MOM_SYSTEM_PROMPT: &str = "You are generating professional Minutes of Meeting (MOM) from a meeting transcript and/or the user's personal notes.\n\nRules you must follow strictly:\n1. Use only information actually present in the supplied transcript and/or notes. Do not invent facts, decisions, participants, owners, or deadlines.\n2. Distinguish clearly between a decision that was actually made or agreed, versus something that was merely suggested or discussed. Do not convert a suggestion (e.g. \"we could...\", \"maybe we should...\") into a decision or a committed action item.\n3. For action items, only include one if a specific commitment was actually made by someone. If an owner or due date is not clearly stated, use the exact text \"Not specified\" rather than guessing.\n4. Do not infer that someone was a participant merely because they were mentioned in conversation (e.g. \"let's ask John about this\" does not mean John attended the meeting).\n5. Do not perform psychological, emotional, or sentiment analysis of any participant, and do not comment on tone, mood, or interpersonal dynamics.\n6. If a section has no real content to report, return an empty list for it rather than inventing filler content. The purpose field should always contain a concise best-effort summary based on whatever material is actually available.\n7. Preserve important technical terms, product names, numbers, and dates exactly as given in the source material.\n8. Keep language concise and professional.";

fn mom_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "title": {"type": "string"},
            "date": {"type": "string"},
            "participants": {"type": "array", "items": {"type": "string"}},
            "purpose": {"type": "string"},
            "discussion_points": {"type": "array", "items": {"type": "string"}},
            "decisions": {"type": "array", "items": {"type": "string"}},
            "action_items": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "action": {"type": "string"},
                        "owner": {"type": "string"},
                        "due_date": {"type": "string"},
                        "status": {"type": "string"}
                    },
                    "required": ["action", "owner", "due_date", "status"]
                }
            },
            "open_questions": {"type": "array", "items": {"type": "string"}},
            "next_steps": {"type": "array", "items": {"type": "string"}}
        },
        "required": [
            "title", "date", "participants", "purpose", "discussion_points",
            "decisions", "action_items", "open_questions", "next_steps"
        ]
    })
}

fn mom_to_markdown(mom: &Mom) -> String {
    let mut md = String::new();
    md.push_str("# Minutes of Meeting\n\n");
    md.push_str(&format!("**Meeting:** {}\n\n", mom.title));
    md.push_str(&format!("**Date:** {}\n\n", mom.date));
    if !mom.participants.is_empty() {
        md.push_str("## Participants\n\n");
        for p in &mom.participants {
            md.push_str(&format!("- {p}\n"));
        }
        md.push('\n');
    }
    md.push_str("## Purpose\n\n");
    md.push_str(&format!("{}\n\n", mom.purpose));
    if !mom.discussion_points.is_empty() {
        md.push_str("## Key Discussion Points\n\n");
        for d in &mom.discussion_points {
            md.push_str(&format!("- {d}\n"));
        }
        md.push('\n');
    }
    if !mom.decisions.is_empty() {
        md.push_str("## Decisions Made\n\n");
        for d in &mom.decisions {
            md.push_str(&format!("- {d}\n"));
        }
        md.push('\n');
    }
    if !mom.action_items.is_empty() {
        md.push_str("## Action Items\n\n");
        md.push_str("| Action | Owner | Due Date | Status |\n|---|---|---|---|\n");
        for a in &mom.action_items {
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                a.action, a.owner, a.due_date, a.status
            ));
        }
        md.push('\n');
    }
    if !mom.open_questions.is_empty() {
        md.push_str("## Open Questions\n\n");
        for q in &mom.open_questions {
            md.push_str(&format!("- {q}\n"));
        }
        md.push('\n');
    }
    if !mom.next_steps.is_empty() {
        md.push_str("## Next Steps\n\n");
        for n in &mom.next_steps {
            md.push_str(&format!("- {n}\n"));
        }
    }
    md
}

fn write_mom_set(root: &PathBuf, mom: &Mom, also_write_generated: bool) -> Result<(), String> {
    let mom_dir = root.join("mom");
    fs::create_dir_all(&mom_dir).map_err(|e| format!("create mom dir: {e}"))?;

    let json = serde_json::to_string_pretty(mom).map_err(|e| format!("serialize mom: {e}"))?;
    let md = mom_to_markdown(mom);

    if also_write_generated {
        fs::write(mom_dir.join("mom_generated.json"), &json)
            .map_err(|e| format!("write mom_generated.json: {e}"))?;
        fs::write(mom_dir.join("mom_generated.md"), &md)
            .map_err(|e| format!("write mom_generated.md: {e}"))?;
    }
    fs::write(mom_dir.join("mom_final.json"), &json)
        .map_err(|e| format!("write mom_final.json: {e}"))?;
    fs::write(mom_dir.join("mom_final.md"), &md)
        .map_err(|e| format!("write mom_final.md: {e}"))?;
    Ok(())
}

async fn call_mom_model(
    app_settings: &settings::AppSettings,
    input: &GenerationInput,
    meeting_id: &str,
) -> Result<Mom, String> {
    let user_content = generation_input::build_user_content(input);
    let raw_json = openai_http::call_chat_json(
        &app_settings.openai_api_key,
        &app_settings.mom_model,
        MOM_SYSTEM_PROMPT,
        &user_content,
        "minutes_of_meeting",
        mom_json_schema(),
    )
    .await?;

    let parsed: MomModelOutput = serde_json::from_str(&raw_json).map_err(|e| {
        format!("could not parse the AI's MOM output - raw output: {raw_json} | parse error: {e}")
    })?;

    Ok(Mom {
        meeting_id: meeting_id.to_string(),
        title: parsed.title,
        date: parsed.date,
        participants: parsed.participants,
        purpose: parsed.purpose,
        discussion_points: parsed.discussion_points,
        decisions: parsed.decisions,
        action_items: parsed.action_items,
        open_questions: parsed.open_questions,
        next_steps: parsed.next_steps,
        model: app_settings.mom_model.clone(),
        created_at: chrono::Local::now().to_rfc3339(),
    })
}

#[tauri::command]
pub async fn generate_mom(meeting_id: String) -> Result<Mom, String> {
    let root = session::find_meeting_root(&meeting_id)?;
    let input = generation_input::build_generation_input(&meeting_id)?;
    if input.transcript_text.is_none() && input.manual_notes.is_none() {
        return Err(
            "No transcript or notes are available for this meeting yet - nothing to generate a MOM from."
                .to_string(),
        );
    }

    let app_settings = settings::load_settings()?;
    if app_settings.openai_api_key.trim().is_empty() {
        return Err(
            "no OpenAI API key is configured - add one in Settings before generating a MOM"
                .to_string(),
        );
    }

    session::update_mom_status(&root, "GENERATING")?;

    let result = call_mom_model(&app_settings, &input, &meeting_id).await;

    match result {
        Ok(mom) => {
            write_mom_set(&root, &mom, true)?;
            session::update_mom_status(&root, "GENERATED")?;
            Ok(mom)
        }
        Err(e) => {
            let _ = session::update_mom_status(&root, "FAILED");
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn regenerate_mom(meeting_id: String) -> Result<Mom, String> {
    generate_mom(meeting_id).await
}

#[tauri::command]
pub fn get_mom(meeting_id: String) -> Result<Mom, String> {
    let root = session::find_meeting_root(&meeting_id)?;
    let content = fs::read_to_string(root.join("mom").join("mom_final.json"))
        .map_err(|e| format!("read mom_final.json: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("parse mom_final.json: {e}"))
}

#[tauri::command]
pub fn get_mom_markdown(meeting_id: String) -> Result<String, String> {
    let root = session::find_meeting_root(&meeting_id)?;
    fs::read_to_string(root.join("mom").join("mom_final.md"))
        .map_err(|e| format!("read mom_final.md: {e}"))
}

#[tauri::command]
pub fn save_mom(meeting_id: String, mom: Mom) -> Result<(), String> {
    let root = session::find_meeting_root(&meeting_id)?;
    write_mom_set(&root, &mom, false)?;
    session::update_mom_status(&root, "EDITED")?;
    Ok(())
}