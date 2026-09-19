//! Device-level WASAPI loopback capture - continuous, stoppable version
//! for the real recorder (adapted from the validated audio_capture_poc).
//! Captures other meeting participants' audio (everything rendered to
//! the default output device) until signaled to stop.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use windows::Win32::Media::Audio::{
    eConsole, eRender, IAudioCaptureClient, IAudioClient, IMMDeviceEnumerator,
    MMDeviceEnumerator, AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED,
    AUDCLNT_STREAMFLAGS_LOOPBACK, WAVEFORMATEX,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED,
};

pub fn run_loopback_capture(
    output_path: String,
    should_stop: Arc<AtomicBool>,
) -> windows::core::Result<()> {
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.ok()?;

    let enumerator: IMMDeviceEnumerator =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
    let device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole)? };
    let audio_client: IAudioClient = unsafe { device.Activate(CLSCTX_ALL, None)? };

    let wf_ptr = unsafe { audio_client.GetMixFormat()? };
    let wf: WAVEFORMATEX = unsafe { *wf_ptr };

    unsafe {
        audio_client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_LOOPBACK,
            10_000_000,
            0,
            wf_ptr,
            None,
        )?;
    }

    let capture_client: IAudioCaptureClient = unsafe { audio_client.GetService()? };

    let spec = hound::WavSpec {
        channels: wf.nChannels,
        sample_rate: wf.nSamplesPerSec,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(&output_path, spec)
        .map_err(|_| windows::core::Error::from_win32())?;

    unsafe { audio_client.Start()? };

    while !should_stop.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(10));
        loop {
            let packet_len = unsafe { capture_client.GetNextPacketSize()? };
            if packet_len == 0 {
                break;
            }

            let mut data_ptr: *mut u8 = std::ptr::null_mut();
            let mut num_frames: u32 = 0;
            let mut flags: u32 = 0;
            unsafe {
                capture_client.GetBuffer(&mut data_ptr, &mut num_frames, &mut flags, None, None)?;
            }

            let is_silent = (flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32) != 0;
            let channels = wf.nChannels as usize;
            let frame_bytes = wf.nBlockAlign as usize;

            if num_frames > 0 {
                if is_silent || data_ptr.is_null() {
                    for _ in 0..(num_frames as usize * channels) {
                        let _ = writer.write_sample(0.0f32);
                    }
                } else {
                    let byte_len = num_frames as usize * frame_bytes;
                    let bytes = unsafe { std::slice::from_raw_parts(data_ptr, byte_len) };
                    for frame in bytes.chunks_exact(frame_bytes) {
                        for ch in frame.chunks_exact(4) {
                            let sample = f32::from_le_bytes([ch[0], ch[1], ch[2], ch[3]]);
                            let _ = writer.write_sample(sample);
                        }
                    }
                }
            }
            unsafe { capture_client.ReleaseBuffer(num_frames)?; }
        }
    }

    unsafe { audio_client.Stop()? };
    let _ = writer.finalize();
    unsafe { CoTaskMemFree(Some(wf_ptr as *const c_void)) };
    Ok(())
}
