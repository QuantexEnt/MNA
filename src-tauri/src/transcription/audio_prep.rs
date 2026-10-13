//! Audio preparation for transcription uploads (Step A: single-request
//! short meetings) and chunking + speaker-reference extraction for long
//! meetings (Step B).
//!
//! Downsampling uses simple decimation (keep every 3rd sample, since
//! 48000 / 16000 = 3 exactly) rather than a proper anti-aliased
//! resample - a known, deliberate simplification (DECISIONS.md #054).
//!
//! No base64 crate is used (keeping dependencies minimal per
//! DECISIONS.md #053) - a small standard RFC 4648 encoder is included
//! below instead, needed only for building known-speaker reference data
//! URLs.

use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

pub struct PreppedAudio {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
}

/// Downmixes the captured 48kHz stereo float meeting.wav to mono and
/// downsamples to 16kHz, returning raw samples in memory so callers can
/// either write the whole thing as one file (short meetings) or split
/// it into chunks (long meetings) without re-reading the source.
pub fn downmix_and_downsample(source_wav_path: &str) -> Result<PreppedAudio, String> {
    let mut reader = WavReader::open(source_wav_path)
        .map_err(|e| format!("open recording for transcription prep: {e}"))?;
    let spec = reader.spec();

    if spec.sample_format != SampleFormat::Float || spec.bits_per_sample != 32 {
        return Err(format!(
            "unexpected recording format for transcription prep: {spec:?} (expected 32-bit float, as produced by merge.rs)"
        ));
    }
    if spec.channels == 0 {
        return Err("recording has zero channels".to_string());
    }

    let channels = spec.channels as usize;
    let raw_samples: Vec<f32> = reader.samples::<f32>().filter_map(|s| s.ok()).collect();

    let frame_count = raw_samples.len() / channels;
    let mut mono: Vec<f32> = Vec::with_capacity(frame_count);
    for frame in 0..frame_count {
        let mut sum = 0.0f32;
        for ch in 0..channels {
            sum += raw_samples[frame * channels + ch];
        }
        mono.push(sum / channels as f32);
    }

    const TARGET_RATE: u32 = 16_000;
    if spec.sample_rate % TARGET_RATE != 0 {
        return Err(format!(
            "source sample rate {}Hz is not a clean multiple of {}Hz - naive decimation would distort pitch; a proper resampler would be needed for this rate",
            spec.sample_rate, TARGET_RATE
        ));
    }
    let decimation_factor = (spec.sample_rate / TARGET_RATE).max(1) as usize;

    let samples: Vec<i16> = mono
        .iter()
        .step_by(decimation_factor)
        .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect();

    Ok(PreppedAudio {
        samples,
        sample_rate: TARGET_RATE,
    })
}

fn write_wav(path: &str, samples: &[i16], sample_rate: u32) -> Result<(), String> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer =
        WavWriter::create(path, spec).map_err(|e| format!("create wav file {path}: {e}"))?;
    for &s in samples {
        writer
            .write_sample(s)
            .map_err(|e| format!("write sample to {path}: {e}"))?;
    }
    writer
        .finalize()
        .map_err(|e| format!("finalize wav file {path}: {e}"))
}

/// Writes the full prepped audio as one WAV file (used for meetings
/// that fit the API's single-request size limit). Returns (size_bytes,
/// duration_seconds).
pub fn write_full(prepped: &PreppedAudio, output_path: &str) -> Result<(u64, f64), String> {
    write_wav(output_path, &prepped.samples, prepped.sample_rate)?;
    let size_bytes = std::fs::metadata(output_path)
        .map_err(|e| format!("stat prepped audio file: {e}"))?
        .len();
    let duration_seconds = prepped.samples.len() as f64 / prepped.sample_rate as f64;
    Ok((size_bytes, duration_seconds))
}

pub struct AudioChunk {
    pub path: String,
    pub start_offset_seconds: f64,
}

/// Splits prepped audio into sequential WAV files, each at most
/// `max_chunk_seconds` long, written into `chunk_dir`.
pub fn split_into_chunks(
    prepped: &PreppedAudio,
    max_chunk_seconds: f64,
    chunk_dir: &str,
) -> Result<Vec<AudioChunk>, String> {
    let max_chunk_samples = (max_chunk_seconds * prepped.sample_rate as f64) as usize;
    if max_chunk_samples == 0 {
        return Err("invalid chunk duration".to_string());
    }

    let mut chunks = Vec::new();
    let mut offset = 0usize;
    let mut index = 0usize;
    while offset < prepped.samples.len() {
        let end = (offset + max_chunk_samples).min(prepped.samples.len());
        let slice = &prepped.samples[offset..end];

        let path = format!("{chunk_dir}/_chunk_{index}.wav");
        write_wav(&path, slice, prepped.sample_rate)?;

        chunks.push(AudioChunk {
            path,
            start_offset_seconds: offset as f64 / prepped.sample_rate as f64,
        });

        offset = end;
        index += 1;
    }

    Ok(chunks)
}

/// Re-reads an already-written mono 16-bit WAV chunk file back into raw
/// samples (needed to slice out a reference clip from it after the API
/// has told us which time ranges belong to which speaker).
pub fn read_i16_mono(path: &str) -> Result<Vec<i16>, String> {
    let mut reader = WavReader::open(path).map_err(|e| format!("open chunk for re-read: {e}"))?;
    Ok(reader.samples::<i16>().filter_map(|s| s.ok()).collect())
}

/// Writes a short slice of samples (2-10 seconds, per OpenAI's
/// known-speaker reference requirement) to a temp WAV file, reads it
/// back as bytes, base64-encodes it as a data URL, and removes the temp
/// file.
pub fn encode_reference_clip(
    samples: &[i16],
    sample_rate: u32,
    temp_path: &str,
) -> Result<String, String> {
    write_wav(temp_path, samples, sample_rate)?;
    let bytes = std::fs::read(temp_path).map_err(|e| format!("read reference clip: {e}"))?;
    let _ = std::fs::remove_file(temp_path);
    Ok(format!("data:audio/wav;base64,{}", base64_encode(&bytes)))
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    let mut chunks = data.chunks_exact(3);
    for chunk in &mut chunks {
        let n = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
        out.push(CHARS[(n >> 18 & 0x3F) as usize] as char);
        out.push(CHARS[(n >> 12 & 0x3F) as usize] as char);
        out.push(CHARS[(n >> 6 & 0x3F) as usize] as char);
        out.push(CHARS[(n & 0x3F) as usize] as char);
    }
    let rem = chunks.remainder();
    match rem.len() {
        1 => {
            let n = (rem[0] as u32) << 16;
            out.push(CHARS[(n >> 18 & 0x3F) as usize] as char);
            out.push(CHARS[(n >> 12 & 0x3F) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let n = ((rem[0] as u32) << 16) | ((rem[1] as u32) << 8);
            out.push(CHARS[(n >> 18 & 0x3F) as usize] as char);
            out.push(CHARS[(n >> 12 & 0x3F) as usize] as char);
            out.push(CHARS[(n >> 6 & 0x3F) as usize] as char);
            out.push('=');
        }
        _ => {}
    }
    out
}