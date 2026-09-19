//! Real dual-source meeting audio recorder (device loopback + gated mic).
//! Built from three separately-validated POCs, composed per the
//! architecture agreed in DECISIONS.md #013-A.
//!
//! Scope note: this module only records to a fixed temp-directory test
//! file for now. It is NOT yet wired to real meeting sessions or local
//! storage (that's Phase 5/6, IMPLEMENTATION_PLAN.md) - deliberately
//! kept narrow so the recorder itself can be proven correct in isolation
//! first, per this project's own incremental-phase discipline.

mod loopback;
mod merge;
mod mic_gated;
mod mute_watcher;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

struct ActiveRecording {
    should_stop: Arc<AtomicBool>,
    loopback_handle: JoinHandle<()>,
    mic_handle: JoinHandle<()>,
    loopback_temp_path: String,
    mic_temp_path: String,
    final_output_path: String,
}

static ACTIVE: Mutex<Option<ActiveRecording>> = Mutex::new(None);

/// Starts recording. Returns the path the final merged file will be
/// written to once stop_recording() is called.
#[tauri::command]
pub fn start_recording() -> Result<String, String> {
    let mut active = ACTIVE
        .lock()
        .map_err(|_| "recording state lock poisoned".to_string())?;
    if active.is_some() {
        return Err("a recording is already active".to_string());
    }

    let temp_dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let final_output_path = temp_dir
        .join(format!("mna_test_recording_{timestamp}.wav"))
        .to_string_lossy()
        .to_string();
    let loopback_temp_path = format!("{final_output_path}.loopback_temp.wav");
    let mic_temp_path = format!("{final_output_path}.mic_temp.wav");

    let should_stop = Arc::new(AtomicBool::new(false));
    // Fail-closed default: mic gate starts CLOSED until the watcher
    // positively confirms "unmuted" (DECISIONS.md #013-A).
    let is_unmuted = Arc::new(AtomicBool::new(false));

    mute_watcher::start_watching(is_unmuted.clone(), should_stop.clone());

    let lb_path = loopback_temp_path.clone();
    let lb_stop = should_stop.clone();
    let loopback_handle = std::thread::spawn(move || {
        if let Err(e) = loopback::run_loopback_capture(lb_path, lb_stop) {
            eprintln!("[audio] loopback capture error: {e:?}");
        }
    });

    let mic_path = mic_temp_path.clone();
    let mic_stop = should_stop.clone();
    let mic_handle = std::thread::spawn(move || {
        if let Err(e) = mic_gated::run_gated_mic_capture(mic_path, is_unmuted, mic_stop) {
            eprintln!("[audio] mic capture error: {e:?}");
        }
    });

    println!("[audio] Recording started. Will save to: {final_output_path}");

    *active = Some(ActiveRecording {
        should_stop,
        loopback_handle,
        mic_handle,
        loopback_temp_path,
        mic_temp_path,
        final_output_path: final_output_path.clone(),
    });

    Ok(final_output_path)
}

#[tauri::command]
pub fn stop_recording() -> Result<String, String> {
    let recording = {
        let mut active = ACTIVE
            .lock()
            .map_err(|_| "recording state lock poisoned".to_string())?;
        active.take()
    };

    let Some(recording) = recording else {
        return Err("no active recording".to_string());
    };

    recording.should_stop.store(true, Ordering::SeqCst);
    let _ = recording.loopback_handle.join();
    let _ = recording.mic_handle.join();

    println!("[audio] Recording stopped. Merging temp files...");

    merge::merge_wav_files(
        &recording.loopback_temp_path,
        &recording.mic_temp_path,
        &recording.final_output_path,
    )?;

    println!(
        "[audio] Merged recording saved to: {}",
        recording.final_output_path
    );

    Ok(recording.final_output_path)
}
