//! Decode an arbitrary user-provided audio *or* video file into 16 kHz mono
//! `f32` PCM, ready to hand to [`crate::managers::transcription::TranscriptionManager::transcribe`].
//!
//! Two strategies are tried, in order:
//!
//! 1. [`symphonia`] — a pure-Rust demuxer/decoder. This covers the vast
//!    majority of audio files (wav, mp3, flac, ogg/vorbis) and audio-only /
//!    mp4-family containers (m4a, most .mp4 videos with an AAC audio track)
//!    without requiring anything to be installed on the user's system.
//! 2. A system `ffmpeg` binary, if one is on `PATH`. This is a fallback for
//!    containers/codecs symphonia does not support (e.g. many `.mkv`,
//!    `.mov`, or `.avi` files, or videos with opus/ac3 audio tracks).
//!    `ffmpeg` already knows how to pull the audio track out of essentially
//!    any video file, so leaning on it here keeps this module small instead
//!    of reimplementing a video demuxer.
//!
//! If neither strategy works we return a clear error explaining both what
//! was tried and, when relevant, that installing `ffmpeg` would help.

use anyhow::{anyhow, Context, Result};
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use super::resampler::FrameResampler;

const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Decode `path` (any audio or video file) down to mono 16 kHz `f32` samples.
pub fn decode_media_file_to_16k_mono<P: AsRef<Path>>(path: P) -> Result<Vec<f32>> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(anyhow!("File not found: {}", path.display()));
    }

    match decode_with_symphonia(path) {
        Ok(samples) if !samples.is_empty() => Ok(samples),
        Ok(_) => {
            // Symphonia opened the file but produced no audio (e.g. a video
            // container it can demux but whose audio codec it can't decode).
            // Fall through to ffmpeg.
            decode_with_ffmpeg(path).with_context(|| {
                format!(
                    "'{}' has no audio symphonia could decode, and the ffmpeg fallback also failed",
                    path.display()
                )
            })
        }
        Err(symphonia_err) => decode_with_ffmpeg(path).with_context(|| {
            format!(
                "Could not decode '{}' with the built-in decoder ({}), and the ffmpeg fallback also failed. \
                 Installing ffmpeg (https://ffmpeg.org/download.html) and making sure it's on PATH \
                 will let Handy transcribe more video/audio formats.",
                path.display(),
                symphonia_err
            )
        }),
    }
}

/// Try decoding purely in Rust via symphonia. Handles common audio formats
/// and mp4/m4a containers directly, with no external dependency.
fn decode_with_symphonia(path: &Path) -> Result<Vec<f32>> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open '{}'", path.display()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .context("Unrecognized or unsupported container format")?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| anyhow!("No decodable audio track found in file"))?
        .clone();
    let track_id = track.id;
    let source_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| anyhow!("Audio track is missing a sample rate"))?;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .context("Unsupported audio codec")?;

    let mut mono_samples: Vec<f32> = Vec::new();
    // Lazily created once we see the first decoded packet's signal spec
    // (channel count / sample rate), as required by symphonia's API.
    let mut sample_buf: Option<SampleBuffer<f32>> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(symphonia::core::errors::Error::ResetRequired) => break,
            Err(e) => return Err(e).context("Error reading packet"),
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                let n_channels = spec.channels.count().max(1);

                if sample_buf.is_none() {
                    sample_buf = Some(SampleBuffer::<f32>::new(decoded.capacity() as u64, spec));
                }
                let buf = sample_buf.as_mut().unwrap();
                buf.copy_interleaved_ref(decoded);

                // Down-mix interleaved [ch0, ch1, ch0, ch1, ...] frames to mono.
                let interleaved = buf.samples();
                mono_samples.reserve(interleaved.len() / n_channels);
                for frame in interleaved.chunks_exact(n_channels) {
                    let sum: f32 = frame.iter().sum();
                    mono_samples.push(sum / n_channels as f32);
                }
            }
            // Non-fatal decode errors (e.g. a corrupt frame) are skipped;
            // one bad packet shouldn't abort transcription of an otherwise
            // fine file.
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(e).context("Fatal decode error"),
        }
    }

    if mono_samples.is_empty() {
        return Ok(Vec::new());
    }

    Ok(resample_to_16k(mono_samples, source_rate))
}

/// Resample mono `f32` samples from `source_rate` to [`TARGET_SAMPLE_RATE`].
fn resample_to_16k(samples: Vec<f32>, source_rate: u32) -> Vec<f32> {
    if source_rate == TARGET_SAMPLE_RATE {
        return samples;
    }

    let mut resampler = FrameResampler::new(
        source_rate as usize,
        TARGET_SAMPLE_RATE as usize,
        Duration::from_secs(1),
    );
    let mut out = Vec::with_capacity(samples.len() * TARGET_SAMPLE_RATE as usize / source_rate.max(1) as usize);
    resampler.push(&samples, |frame| out.extend_from_slice(frame));
    resampler.finish(|frame| out.extend_from_slice(frame));
    out
}

/// Fallback path: shell out to a system `ffmpeg` to convert *anything* it
/// understands (essentially all audio/video formats in the wild) into raw
/// 16 kHz mono 16-bit PCM on stdout, then parse that.
fn decode_with_ffmpeg(path: &Path) -> Result<Vec<f32>> {
    let mut child = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
        ])
        .arg(path)
        .args([
            "-vn", // no video
            "-ac", "1", // mono
            "-ar", "16000", // 16kHz
            "-f", "s16le", // raw signed 16-bit little-endian PCM
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context(
            "ffmpeg is not installed or not on PATH. Install ffmpeg to transcribe this file \
             (https://ffmpeg.org/download.html).",
        )?;

    let mut raw = Vec::new();
    child
        .stdout
        .take()
        .expect("stdout was piped")
        .read_to_end(&mut raw)
        .context("Failed to read ffmpeg output")?;

    let status = child.wait().context("Failed to wait on ffmpeg")?;
    if !status.success() {
        let mut stderr = String::new();
        if let Some(mut e) = child.stderr.take() {
            let _ = e.read_to_string(&mut stderr);
        }
        return Err(anyhow!("ffmpeg failed to decode file: {}", stderr.trim()));
    }

    // Convert little-endian i16 PCM bytes to f32 in [-1.0, 1.0].
    let samples = raw
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / i16::MAX as f32)
        .collect();

    Ok(samples)
}
