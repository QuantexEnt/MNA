//! Phase 9: Transcription (real mode only).
//!
//! Step A (verified working): single-request transcription for
//! meetings whose prepped audio fits OpenAI's 25MB/request limit
//! (~13 minutes at 16kHz mono 16-bit, uncompressed).
//!
//! Step B (this version): meetings over that limit are automatically
//! split into sequential ~10-minute chunks. The first chunk is
//! transcribed normally (generic "A"/"B"/"C" labels). For each
//! subsequent chunk, up to 4 previously-seen speakers are "pinned" by
//! auto-extracting a short reference clip from their own earlier audio
//! (via OpenAI's known_speaker_names[]/known_speaker_references[]
//! feature) and passing it along - so the same real person keeps the
//! same letter for the whole meeting, with zero manual sample input
//! required. New speakers who join mid-meeting are picked up and
//! pinned forward too, as capacity allows (max 4 pinned speakers total,
//! an OpenAI-side limit). All chunk results are reassembled into one
//! continuous transcript with correctly offset timestamps.
//!
//! Known limitations, documented rather than hidden:
//! - More than 4 distinct speakers in one meeting: the 5th+ person may
//!   get an inconsistent label across chunks (API-side cap, not
//!   something we can work around).
//! - If any single chunk fails after its retries, the entire
//!   transcription attempt fails - there is no partial-save/resume yet.
//!   For a long meeting this means a late failure forfeits earlier
//!   successful (and billed) chunk calls. Worth revisiting if real
//!   testing shows this happens often.
//! - The known_speaker_references mechanism itself is not something
//!   this implementation has been validated against beyond OpenAI's
//!   documented examples - your first long real-meeting test is the
//!   actual validation of whether cross-chunk pinning behaves as
//!   documented.

mod audio_prep;

use crate::session::{self, MeetingMetadata};
use crate::settings;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const MAX_UPLOAD_BYTES: u64 = 25 * 1024 * 1024; // OpenAI's hard per-request cap
const MAX_CHUNK_SECONDS: f64 = 600.0; // 10 minutes per chunk (~19.2MB, safe margin under the cap)
const OPENAI_TRANSCRIBE_URL: &str = "https://api.openai.com/v1/audio/transcriptions";
const OPENAI_MODELS_URL: &str = "https://api.openai.com/v1/models";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub speaker: String,
    pub text: String,
    pub start: Option<f64>,
    pub end: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub meeting_id: String,
    pub text: String,
    pub language: String,
    pub duration_seconds: Option<f64>,
    pub model: String,
    pub segments: Vec<TranscriptSegment>,
    pub created_at: String,
}

// --- Raw API response shapes, per OpenAI's confirmed diarized_json
// example (duration can be absent in practice; `language` is an
// opportunistic capture - not confirmed present in every response
// variant, so it stays optional and defaults to "unknown" if missing) ---

#[derive(Debug, Deserialize)]
struct ApiSegment {
    start: f64,
    end: f64,
    text: String,
    speaker: String,
}

#[derive(Debug, Deserialize)]
struct ApiDiarizedResponse {
    text: String,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    segments: Vec<ApiSegment>,
    #[serde(default)]
    language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    message: String,
}

fn transcript_json_path(root: &PathBuf) -> PathBuf {
    root.join("transcript").join("transcript.json")
}
fn transcript_md_path(root: &PathBuf) -> PathBuf {
    root.join("transcript").join("transcript.md")
}
fn speakers_json_path(root: &PathBuf) -> PathBuf {
    root.join("transcript").join("speakers.json")
}

fn write_transcript_files(root: &PathBuf, transcript: &Transcript) -> Result<(), String> {
    let json = serde_json::to_string_pretty(transcript)
        .map_err(|e| format!("serialize transcript.json: {e}"))?;
    fs::write(transcript_json_path(root), json)
        .map_err(|e| format!("write transcript.json: {e}"))?;

    let mut md = String::new();
    md.push_str("# Meeting Transcript\n\n");
    md.push_str(&format!("**Model:** {}\n\n", transcript.model));
    md.push_str(&format!("**Detected language:** {}\n\n", transcript.language));
    md.push_str("## Transcript\n\n");
    for seg in &transcript.segments {
        md.push_str(&format!("**{}:** {}\n\n", seg.speaker, seg.text));
    }
    fs::write(transcript_md_path(root), md).map_err(|e| format!("write transcript.md: {e}"))?;

    Ok(())
}

fn load_stored_transcript(root: &PathBuf) -> Result<Transcript, String> {
    let content = fs::read_to_string(transcript_json_path(root))
        .map_err(|e| format!("read transcript.json: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("parse transcript.json: {e}"))
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))
}

/// Single raw API call. `known_speakers` maps a speaker label (e.g.
/// "A") to its reference data URL; pass an empty map for no pinning
/// (used for the very first chunk, and for short unchunked meetings).
/// Capped internally at 4 entries - OpenAI's documented maximum.
async fn call_openai_transcribe_raw(
    api_key: &str,
    model: &str,
    audio_path: &str,
    known_speakers: &HashMap<String, String>,
) -> Result<ApiDiarizedResponse, String> {
    let file_bytes = fs::read(audio_path).map_err(|e| format!("read audio for upload: {e}"))?;
    let file_part = multipart::Part::bytes(file_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("build upload part: {e}"))?;

    let mut form = multipart::Form::new()
        .text("model", model.to_string())
        .text("response_format", "diarized_json")
        // Required by OpenAI whenever input audio exceeds 30 seconds.
        .text("chunking_strategy", "auto")
        .part("file", file_part);

    for (name, reference) in known_speakers.iter().take(4) {
        form = form.text("known_speaker_names[]", name.clone());
        form = form.text("known_speaker_references[]", reference.clone());
    }
    // Deliberately no `language` field - letting the model auto-detect.

    let client = http_client()?;
    let response = client
        .post(OPENAI_TRANSCRIBE_URL)
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("network error calling OpenAI: {e}"))?;

    let status = response.status();
    let body_text = response
        .text()
        .await
        .map_err(|e| format!("read OpenAI response body: {e}"))?;

    if !status.is_success() {
        if let Ok(err_body) = serde_json::from_str::<ApiErrorBody>(&body_text) {
            return Err(format!("OpenAI API error ({status}): {}", err_body.error.message));
        }
        return Err(format!("OpenAI API error ({status}): {body_text}"));
    }

    serde_json::from_str::<ApiDiarizedResponse>(&body_text).map_err(|e| {
        format!(
            "could not parse OpenAI's response - the response shape may differ from what was expected. Raw response: {body_text} | parse error: {e}"
        )
    })
}

/// Wraps the raw call with a small retry (2 attempts, 2s delay) for
/// transient failures. Not selective about error type for V1 simplicity
/// (DECISIONS.md #054) - retries any failure once.
async fn call_with_retry(
    api_key: &str,
    model: &str,
    audio_path: &str,
    known_speakers: &HashMap<String, String>,
) -> Result<ApiDiarizedResponse, String> {
    const MAX_ATTEMPTS: u32 = 2;
    let mut last_err = String::new();
    for attempt in 1..=MAX_ATTEMPTS {
        match call_openai_transcribe_raw(api_key, model, audio_path, known_speakers).await {
            Ok(resp) => return Ok(resp),
            Err(e) => {
                eprintln!("[transcription] attempt {attempt}/{MAX_ATTEMPTS} failed: {e}");
                last_err = e;
                if attempt < MAX_ATTEMPTS {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    }
    Err(last_err)
}

#[tauri::command]
pub async fn transcribe_meeting(meeting_id: String) -> Result<Transcript, String> {
    let root = session::find_meeting_root(&meeting_id)?;
    let metadata: MeetingMetadata = session::read_meeting_metadata(&root)?;

    if metadata.transcript_status == "TRANSCRIBED" {
        return load_stored_transcript(&root);
    }

    let Some(audio_info) = metadata.audio.as_ref() else {
        return Err("this meeting has no recorded audio to transcribe".to_string());
    };

    let app_settings = settings::load_settings()?;
    if app_settings.openai_api_key.trim().is_empty() {
        return Err(
            "no OpenAI API key is configured - add one in Settings before transcribing"
                .to_string(),
        );
    }

    session::update_transcript_status(&root, "PROCESSING")?;

    let result: Result<Transcript, String> = async {
        let source_audio_path = root.join(&audio_info.path);
        let transcript_dir = root.join("transcript");
        let transcript_dir_str = transcript_dir
            .to_str()
            .ok_or("invalid transcript dir path")?;

        let prepped = audio_prep::downmix_and_downsample(
            source_audio_path.to_str().ok_or("invalid audio path")?,
        )?;

        let total_bytes_estimate = prepped.samples.len() as u64 * 2 + 44;

        if total_bytes_estimate <= MAX_UPLOAD_BYTES {
            // Short meeting - single request, no chunking needed.
            let full_path = transcript_dir.join("_upload_prepped.wav");
            let full_path_str = full_path.to_str().ok_or("invalid output path")?;
            audio_prep::write_full(&prepped, full_path_str)?;

            let empty_known: HashMap<String, String> = HashMap::new();
            let api_response = call_with_retry(
                &app_settings.openai_api_key,
                &app_settings.transcription_model,
                full_path_str,
                &empty_known,
            )
            .await?;

            let _ = fs::remove_file(&full_path);

            let segments: Vec<TranscriptSegment> = api_response
                .segments
                .into_iter()
                .map(|s| TranscriptSegment {
                    speaker: s.speaker,
                    text: s.text,
                    start: Some(s.start),
                    end: Some(s.end),
                })
                .collect();

            return Ok(Transcript {
                meeting_id: meeting_id.clone(),
                text: api_response.text,
                language: api_response.language.unwrap_or_else(|| "unknown".to_string()),
                duration_seconds: api_response.duration,
                model: app_settings.transcription_model.clone(),
                segments,
                created_at: chrono::Local::now().to_rfc3339(),
            });
        }

        // Long meeting - chunk, with cross-chunk speaker pinning.
        let audio_chunks =
            audio_prep::split_into_chunks(&prepped, MAX_CHUNK_SECONDS, transcript_dir_str)?;

        let mut pinned: HashMap<String, String> = HashMap::new();
        let mut all_segments: Vec<TranscriptSegment> = Vec::new();
        let mut all_text_parts: Vec<String> = Vec::new();
        let mut detected_language: Option<String> = None;

        for (i, chunk) in audio_chunks.iter().enumerate() {
            let known_for_this_chunk = if i == 0 { HashMap::new() } else { pinned.clone() };

            let api_response = call_with_retry(
                &app_settings.openai_api_key,
                &app_settings.transcription_model,
                &chunk.path,
                &known_for_this_chunk,
            )
            .await?;

            if detected_language.is_none() {
                detected_language = api_response.language.clone();
            }

            // Pin any newly-seen speaker, using THIS chunk's own audio,
            // as long as we haven't hit OpenAI's 4-speaker cap.
            if pinned.len() < 4 {
                if let Ok(chunk_samples) = audio_prep::read_i16_mono(&chunk.path) {
                    for seg in &api_response.segments {
                        if pinned.len() >= 4 {
                            break;
                        }
                        if pinned.contains_key(&seg.speaker) {
                            continue;
                        }
                        let dur = seg.end - seg.start;
                        if dur < 1.5 {
                            continue; // too short for a reliable reference
                        }
                        let clip_dur = dur.min(8.0);
                        let mid = (seg.start + seg.end) / 2.0;
                        let clip_start = (mid - clip_dur / 2.0).max(seg.start);
                        let clip_end = (clip_start + clip_dur).min(seg.end);
                        let start_sample =
                            (clip_start * prepped.sample_rate as f64).round() as usize;
                        let end_sample = ((clip_end * prepped.sample_rate as f64).round()
                            as usize)
                            .min(chunk_samples.len());
                        if start_sample >= end_sample {
                            continue;
                        }
                        let temp_ref_path =
                            format!("{transcript_dir_str}/_ref_{}.wav", seg.speaker);
                        if let Ok(data_url) = audio_prep::encode_reference_clip(
                            &chunk_samples[start_sample..end_sample],
                            prepped.sample_rate,
                            &temp_ref_path,
                        ) {
                            pinned.insert(seg.speaker.clone(), data_url);
                        }
                    }
                }
            }

            for seg in api_response.segments {
                all_text_parts.push(format!("{}: {}", seg.speaker, seg.text));
                all_segments.push(TranscriptSegment {
                    speaker: seg.speaker,
                    text: seg.text,
                    start: Some(seg.start + chunk.start_offset_seconds),
                    end: Some(seg.end + chunk.start_offset_seconds),
                });
            }

            let _ = fs::remove_file(&chunk.path);
        }

        Ok(Transcript {
            meeting_id: meeting_id.clone(),
            text: all_text_parts.join("\n\n"),
            language: detected_language.unwrap_or_else(|| "unknown".to_string()),
            duration_seconds: Some(prepped.samples.len() as f64 / prepped.sample_rate as f64),
            model: app_settings.transcription_model.clone(),
            segments: all_segments,
            created_at: chrono::Local::now().to_rfc3339(),
        })
    }
    .await;

    match result {
        Ok(transcript) => {
            write_transcript_files(&root, &transcript)?;
            session::update_transcript_status(&root, "TRANSCRIBED")?;
            Ok(transcript)
        }
        Err(e) => {
            let _ = session::update_transcript_status(&root, "FAILED");
            Err(e)
        }
    }
}

#[tauri::command]
pub fn get_transcript(meeting_id: String) -> Result<Transcript, String> {
    let root = session::find_meeting_root(&meeting_id)?;
    load_stored_transcript(&root)
}

#[tauri::command]
pub fn save_speaker_names(
    meeting_id: String,
    names: HashMap<String, String>,
) -> Result<(), String> {
    let root = session::find_meeting_root(&meeting_id)?;
    let json = serde_json::to_string_pretty(&names)
        .map_err(|e| format!("serialize speakers.json: {e}"))?;
    fs::write(speakers_json_path(&root), json).map_err(|e| format!("write speakers.json: {e}"))
}

#[tauri::command]
pub fn get_speaker_names(meeting_id: String) -> Result<HashMap<String, String>, String> {
    let root = session::find_meeting_root(&meeting_id)?;
    let path = speakers_json_path(&root);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("read speakers.json: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("parse speakers.json: {e}"))
}

#[tauri::command]
pub async fn test_openai_connection(api_key: String) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("no API key provided".to_string());
    }
    let client = http_client()?;
    let response = client
        .get(OPENAI_MODELS_URL)
        .bearer_auth(&api_key)
        .send()
        .await
        .map_err(|e| format!("network error: {e}"))?;

    let status = response.status();
    if status.is_success() {
        Ok("Connection successful".to_string())
    } else {
        let body_text = response.text().await.unwrap_or_default();
        if let Ok(err_body) = serde_json::from_str::<ApiErrorBody>(&body_text) {
            Err(format!("Connection failed: {}", err_body.error.message))
        } else {
            Err(format!("Connection failed ({status})"))
        }
    }
}