//! Meeting session management (Phase 5) + local storage layout (Phase 6).
//! Per LOCAL_STORAGE_SPECIFICATION.md §5/§8 - one folder per meeting
//! containing meeting.json plus audio/, transcript/, notes/, mom/,
//! ai_notes/ subfolders.
//!
//! Storage location: resolved via paths::meetings_root(), which uses
//! Tauri's runtime app-local-data directory - correct per-machine,
//! unlike the previous CARGO_MANIFEST_DIR-based dev-only approach.
//!
//! Title handling: no UI exists yet to enter a real meeting title, so
//! per NOTES_AND_MOM_SPECIFICATION.md §11's fallback, a generic
//! "Meeting" title is used.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfo {
    pub path: String,
    pub duration_seconds: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingMetadata {
    pub id: String,
    pub title: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: String,
    pub audio: Option<AudioInfo>,
    #[serde(default = "default_transcript_status")]
    pub transcript_status: String,
    #[serde(default = "default_generation_status")]
    pub mom_status: String,
    #[serde(default = "default_generation_status")]
    pub ai_notes_status: String,
}

fn default_transcript_status() -> String {
    "NOT_TRANSCRIBED".to_string()
}

fn default_generation_status() -> String {
    "NOT_GENERATED".to_string()
}

struct ActiveSession {
    metadata: MeetingMetadata,
    root: PathBuf,
}

static CURRENT: Mutex<Option<ActiveSession>> = Mutex::new(None);

fn write_meeting_json(root: &PathBuf, metadata: &MeetingMetadata) -> Result<(), String> {
    let json_path = root.join("meeting.json");
    let json = serde_json::to_string_pretty(metadata)
        .map_err(|e| format!("serialize meeting.json: {e}"))?;
    fs::write(&json_path, json).map_err(|e| format!("write meeting.json: {e}"))
}

/// Creates a new meeting session: unique ID, a proper folder with the
/// standard subfolders, and an initial meeting.json with status ACTIVE.
pub fn create_session() -> Result<PathBuf, String> {
    let mut current = CURRENT
        .lock()
        .map_err(|_| "session lock poisoned".to_string())?;
    if current.is_some() {
        return Err("a meeting session is already active".to_string());
    }

    let now = chrono::Local::now();
    let id = format!("meeting_{}", now.timestamp());
    let folder_name = format!("{}_Meeting", now.format("%Y-%m-%d_%H%M%S"));

    let root = crate::paths::meetings_root().join(&folder_name);

    fs::create_dir_all(root.join("audio")).map_err(|e| format!("create audio dir: {e}"))?;
    fs::create_dir_all(root.join("transcript"))
        .map_err(|e| format!("create transcript dir: {e}"))?;
    fs::create_dir_all(root.join("notes")).map_err(|e| format!("create notes dir: {e}"))?;
    fs::create_dir_all(root.join("mom")).map_err(|e| format!("create mom dir: {e}"))?;
    fs::create_dir_all(root.join("ai_notes")).map_err(|e| format!("create ai_notes dir: {e}"))?;

    let metadata = MeetingMetadata {
        id,
        title: "Meeting".to_string(),
        started_at: now.to_rfc3339(),
        ended_at: None,
        status: "ACTIVE".to_string(),
        audio: None,
        transcript_status: default_transcript_status(),
        mom_status: default_generation_status(),
        ai_notes_status: default_generation_status(),
    };

    write_meeting_json(&root, &metadata)?;

    let audio_path = root.join("audio").join("meeting.wav");
    *current = Some(ActiveSession { metadata, root });

    Ok(audio_path)
}

/// Finalizes the current session: updates meeting.json with ended_at,
/// COMPLETED status, and the audio path.
pub fn finish_session(audio_relative_path: &str) -> Result<String, String> {
    let mut current = CURRENT
        .lock()
        .map_err(|_| "session lock poisoned".to_string())?;
    let Some(session) = current.take() else {
        return Err("no active meeting session".to_string());
    };

    let mut metadata = session.metadata;
    metadata.ended_at = Some(chrono::Local::now().to_rfc3339());
    metadata.status = "COMPLETED".to_string();
    metadata.audio = Some(AudioInfo {
        path: audio_relative_path.to_string(),
        duration_seconds: None,
    });

    write_meeting_json(&session.root, &metadata)?;

    Ok(session.root.to_string_lossy().to_string())
}

#[tauri::command]
pub fn save_notes(content: String) -> Result<(), String> {
    let current = CURRENT
        .lock()
        .map_err(|_| "session lock poisoned".to_string())?;
    let Some(session) = current.as_ref() else {
        return Err("no active meeting session".to_string());
    };
    let notes_path = session.root.join("notes").join("notes.md");
    fs::write(&notes_path, content).map_err(|e| format!("write notes.md: {e}"))
}

#[tauri::command]
pub fn list_meetings() -> Result<Vec<MeetingMetadata>, String> {
    let meetings_root = crate::paths::meetings_root();

    if !meetings_root.exists() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    let entries = fs::read_dir(&meetings_root).map_err(|e| format!("read meetings dir: {e}"))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let meeting_json = path.join("meeting.json");
            if meeting_json.exists() {
                if let Ok(content) = fs::read_to_string(&meeting_json) {
                    if let Ok(metadata) = serde_json::from_str::<MeetingMetadata>(&content) {
                        results.push(metadata);
                    }
                }
            }
        }
    }

    results.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    Ok(results)
}

pub fn find_meeting_root(meeting_id: &str) -> Result<PathBuf, String> {
    let meetings_root = crate::paths::meetings_root();

    if !meetings_root.exists() {
        return Err(format!("meeting '{meeting_id}' not found"));
    }

    let entries = fs::read_dir(&meetings_root).map_err(|e| format!("read meetings dir: {e}"))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let meeting_json = path.join("meeting.json");
            if meeting_json.exists() {
                if let Ok(content) = fs::read_to_string(&meeting_json) {
                    if let Ok(metadata) = serde_json::from_str::<MeetingMetadata>(&content) {
                        if metadata.id == meeting_id {
                            return Ok(path);
                        }
                    }
                }
            }
        }
    }

    Err(format!("meeting '{meeting_id}' not found"))
}

pub fn read_meeting_metadata(root: &PathBuf) -> Result<MeetingMetadata, String> {
    let json_path = root.join("meeting.json");
    let content = fs::read_to_string(&json_path).map_err(|e| format!("read meeting.json: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("parse meeting.json: {e}"))
}

pub fn update_transcript_status(root: &PathBuf, status: &str) -> Result<(), String> {
    let mut metadata = read_meeting_metadata(root)?;
    metadata.transcript_status = status.to_string();
    write_meeting_json(root, &metadata)
}

pub fn update_mom_status(root: &PathBuf, status: &str) -> Result<(), String> {
    let mut metadata = read_meeting_metadata(root)?;
    metadata.mom_status = status.to_string();
    write_meeting_json(root, &metadata)
}

pub fn update_ai_notes_status(root: &PathBuf, status: &str) -> Result<(), String> {
    let mut metadata = read_meeting_metadata(root)?;
    metadata.ai_notes_status = status.to_string();
    write_meeting_json(root, &metadata)
}

#[tauri::command]
pub fn get_notes(meeting_id: String) -> Result<String, String> {
    let root = find_meeting_root(&meeting_id)?;
    let notes_path = root.join("notes").join("notes.md");
    if !notes_path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(&notes_path).map_err(|e| format!("read notes.md: {e}"))
}