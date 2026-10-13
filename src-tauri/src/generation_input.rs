//! Shared input-gathering for MOM and AI Notes generation: assembles
//! transcript (with speaker names applied) + manual notes + basic
//! meeting metadata into one common structure, so both generation
//! services build from the same consistently-prepared source material
//! (OPENAI_INTEGRATION.md §16).

use crate::session;
use crate::transcription;
use std::collections::HashMap;

pub struct GenerationInput {
    pub meeting_title: String,
    pub meeting_date: String,
    pub transcript_text: Option<String>,
    pub manual_notes: Option<String>,
}

fn apply_speaker_names(
    transcript: &transcription::Transcript,
    names: &HashMap<String, String>,
) -> String {
    transcript
        .segments
        .iter()
        .map(|seg| {
            let label = if seg.speaker == "You" {
                "You".to_string()
            } else {
                names
                    .get(&seg.speaker)
                    .filter(|n| !n.trim().is_empty())
                    .cloned()
                    .unwrap_or_else(|| seg.speaker.clone())
            };
            format!("{label}: {}", seg.text)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Gathers everything needed to generate a MOM or AI Notes for a
/// meeting. A meeting with no transcript and no notes is a valid
/// (empty) result - callers decide whether to reject that
/// (NOTES_AND_MOM_SPECIFICATION.md §36: don't call the model for empty
/// input).
pub fn build_generation_input(meeting_id: &str) -> Result<GenerationInput, String> {
    let root = session::find_meeting_root(meeting_id)?;
    let metadata = session::read_meeting_metadata(&root)?;

    let transcript_text = if metadata.transcript_status == "TRANSCRIBED" {
        let transcript = transcription::get_transcript(meeting_id.to_string())?;
        let names = transcription::get_speaker_names(meeting_id.to_string())?;
        Some(apply_speaker_names(&transcript, &names))
    } else {
        None
    };

    let notes_content = session::get_notes(meeting_id.to_string())?;
    let manual_notes = if notes_content.trim().is_empty() {
        None
    } else {
        Some(notes_content)
    };

    Ok(GenerationInput {
        meeting_title: metadata.title,
        meeting_date: metadata.started_at,
        transcript_text,
        manual_notes,
    })
}

/// Builds the user-message text sent to the model: meeting metadata
/// plus whichever of transcript/manual notes are actually available.
/// Shared between MOM and AI Notes generation since both want the same
/// source material in the same format.
pub fn build_user_content(input: &GenerationInput) -> String {
    let mut s = String::new();
    s.push_str(&format!("Meeting Title: {}\n", input.meeting_title));
    s.push_str(&format!("Meeting Date: {}\n\n", input.meeting_date));
    if let Some(t) = &input.transcript_text {
        s.push_str("=== Meeting Transcript ===\n");
        s.push_str(t);
        s.push_str("\n\n");
    }
    if let Some(n) = &input.manual_notes {
        s.push_str("=== User's Personal Notes (taken during the meeting) ===\n");
        s.push_str(n);
        s.push('\n');
    }
    s
}