//! Merge the two temp WAV files (loopback + gated mic) into one combined
//! recording. Simple sample-by-sample sum with clamping - an accepted
//! simplification per DECISIONS.md #054 (no premature optimization).
//!
//! A true real-time single-pass mixer would need to solve independent
//! clock drift between two separate audio devices (loopback and capture
//! endpoints can run on different hardware clocks). This offline
//! approach sidesteps that by relying on both threads starting/stopping
//! at approximately the same wall-clock time - a few milliseconds of
//! drift is inaudible and acceptable for meeting transcription purposes.
//! If long-recording drift turns out to matter in practice, revisit
//! this rather than silently trusting it forever.

use std::fs;

pub fn merge_wav_files(
    loopback_path: &str,
    mic_path: &str,
    output_path: &str,
) -> Result<(), String> {
    let mut loopback_reader =
        hound::WavReader::open(loopback_path).map_err(|e| format!("open loopback temp file: {e}"))?;
    let mut mic_reader =
        hound::WavReader::open(mic_path).map_err(|e| format!("open mic temp file: {e}"))?;

    let spec = loopback_reader.spec();
    let mic_spec = mic_reader.spec();
    if spec.sample_rate != mic_spec.sample_rate || spec.channels != mic_spec.channels {
        return Err(format!(
            "format mismatch: loopback {spec:?} vs mic {mic_spec:?} - cannot merge directly"
        ));
    }

    let loopback_samples: Vec<f32> = loopback_reader
        .samples::<f32>()
        .filter_map(|s| s.ok())
        .collect();
    let mic_samples: Vec<f32> = mic_reader.samples::<f32>().filter_map(|s| s.ok()).collect();

    let len = loopback_samples.len().max(mic_samples.len());

    let mut writer =
        hound::WavWriter::create(output_path, spec).map_err(|e| format!("create output: {e}"))?;

    for i in 0..len {
        let a = loopback_samples.get(i).copied().unwrap_or(0.0);
        let b = mic_samples.get(i).copied().unwrap_or(0.0);
        let mixed = (a + b).clamp(-1.0, 1.0);
        writer
            .write_sample(mixed)
            .map_err(|e| format!("write sample: {e}"))?;
    }

    writer.finalize().map_err(|e| format!("finalize output: {e}"))?;

    // Clean up temp files now that they're merged into the final file.
    let _ = fs::remove_file(loopback_path);
    let _ = fs::remove_file(mic_path);

    Ok(())
}
