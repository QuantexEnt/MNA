//! Real dual-source meeting audio recorder (device loopback + gated mic).
//! Built from three separately-validated POCs, composed per the
//! architecture agreed in DECISIONS.md #013-A.
//!
//! Storage: audio path now comes from session.rs (Phase 5/6), which
//! creates the proper per-meeting folder structure. This module no
//! longer computes its own output location.

mod loopback;
mod merge;
mod mic_gated;
mod mute_watcher;

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use windows::Win32::Media::Audio::{
    eConsole, eRender, IAudioClient, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED,
};

struct ActiveRecording {
    should_stop: Arc<AtomicBool>,
    loopback_handle: JoinHandle<()>,
    mic_handle: JoinHandle<()>,
    loopback_temp_path: String,
    mic_temp_path: String,
    final_output_path: String,
}

static ACTIVE: Mutex<Option<ActiveRecording>> = Mutex::new(None);

/// Quickly queries the render (output) device's native mix format, so
/// the mic capture thread can be told to explicitly match it. Runs on
/// its own thread - required, since Tauri's own thread already has COM
/// initialized in a different mode (see project history: RPC_E_CHANGED_MODE).
fn probe_render_format() -> Result<(u16, u32, u16), String> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)
            .ok()
            .map_err(|e| e.to_string())?;
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| e.to_string())?;
        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .map_err(|e| e.to_string())?;
        let audio_client: IAudioClient =
            device.Activate(CLSCTX_ALL, None).map_err(|e| e.to_string())?;
        let wf_ptr = audio_client.GetMixFormat().map_err(|e| e.to_string())?;
        let wf = *wf_ptr;
        let result = (wf.nChannels, wf.nSamplesPerSec, wf.wBitsPerSample);
        CoTaskMemFree(Some(wf_ptr as *const c_void));
        Ok(result)
    }
}

#[tauri::command]
pub fn start_recording() -> Result<String, String> {
    let mut active = ACTIVE
        .lock()
        .map_err(|_| "recording state lock poisoned".to_string())?;
    if active.is_some() {
        return Err("a recording is already active".to_string());
    }

    // Phase 5/6: session.rs owns the folder structure and target path.
    let audio_path = crate::session::create_session()?;
    let final_output_path = audio_path.to_string_lossy().to_string();
    let loopback_temp_path = format!("{final_output_path}.loopback_temp.wav");
    let mic_temp_path = format!("{final_output_path}.mic_temp.wav");

    let (channels, sample_rate, bits) = std::thread::spawn(probe_render_format)
        .join()
        .map_err(|_| "format probe thread panicked".to_string())??;
    println!("[audio] Target format (from render device): {channels}ch {sample_rate}Hz {bits}bit");

    let should_stop = Arc::new(AtomicBool::new(false));
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
        if let Err(e) = mic_gated::run_gated_mic_capture(
            mic_path, is_unmuted, mic_stop, channels, sample_rate, bits,
        ) {
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
        final_output_path,
    });

    Ok(audio_path.to_string_lossy().to_string())
}

fn do_stop(recording: ActiveRecording) -> Result<String, String> {
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

    do_stop(recording)?;
    crate::session::finish_session("audio/meeting.wav")
}

pub fn stop_recording_if_active() -> Option<Result<String, String>> {
    let recording = {
        let mut active = ACTIVE.lock().ok()?;
        active.take()
    }?;

    let result = do_stop(recording).and_then(|_| crate::session::finish_session("audio/meeting.wav"));
    Some(result)
}
