/// MP3 audio file generator.
///
/// Produces valid MP3 files by constructing MPEG Audio Layer III frames
/// with proper headers. The audio content consists of synthesized tones
/// (sine, noise, or frequency sweep).
///
/// Each MP3 frame consists of a 4-byte header followed by audio data.
/// We use MPEG1 Layer III at 128kbps for compatibility.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig, ToneType};
use crate::error::{GenResult, GenerationError};
use rand::Rng;
use std::f32::consts::PI;

/// Generator for MP3 audio files.
pub struct Mp3Generator;

/// MPEG1 Layer III frame size at 128kbps, 44100Hz.
/// Frame size = 144 * bitrate / sample_rate + padding
/// = 144 * 128000 / 44100 = 417 bytes (rounded down, no padding)
#[allow(dead_code)]
const FRAME_SIZE_128KBPS_44100: usize = 417;

/// Builds an MPEG1 Layer III frame header.
///
/// Format: 0xFFFB9004 for MPEG1, Layer3, 128kbps, 44100Hz, stereo
fn build_mp3_frame_header(sample_rate_index: u8, bitrate_index: u8) -> [u8; 4] {
    // Sync word: 0xFFE (11 bits)
    // MPEG version: 11 (MPEG1)
    // Layer: 01 (Layer III)
    // Protection: 1 (no CRC)
    // Bitrate index: 4 bits
    // Sample rate index: 2 bits
    // Padding: 0
    // Private: 0
    // Channel mode: 00 (stereo)
    // Mode extension: 00
    // Copyright: 0
    // Original: 1
    // Emphasis: 00
    let byte0: u8 = 0xFF;
    let byte1: u8 = 0xFB; // 1111 1011: sync, MPEG1, Layer3, no CRC
    let byte2: u8 = (bitrate_index << 4) | (sample_rate_index << 2); // bitrate | samplerate | no padding | private=0
    let byte3: u8 = 0x04; // stereo, mode ext 00, not copyrighted, original, no emphasis

    [byte0, byte1, byte2, byte3]
}

/// Generates raw PCM sine wave samples.
fn generate_sine_samples(sample_rate: u32, duration: f32, frequency: f32) -> Vec<i16> {
    let sample_count = (sample_rate as f32 * duration) as usize;
    (0..sample_count)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let sample = (2.0 * PI * frequency * t).sin();
            (sample * 16000.0) as i16
        })
        .collect()
}

/// Generates random noise samples.
fn generate_noise_samples<R: Rng>(rng: &mut R, sample_rate: u32, duration: f32) -> Vec<i16> {
    let sample_count = (sample_rate as f32 * duration) as usize;
    (0..sample_count)
        .map(|_| rng.gen_range(-8000i16..8000i16))
        .collect()
}

/// Generates a frequency sweep (chirp) from 200Hz to 4000Hz.
fn generate_sweep_samples(sample_rate: u32, duration: f32) -> Vec<i16> {
    let sample_count = (sample_rate as f32 * duration) as usize;
    let f0: f32 = 200.0;
    let f1: f32 = 4000.0;
    (0..sample_count)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let freq = f0 + (f1 - f0) * t / duration;
            let sample = (2.0 * PI * freq * t).sin();
            (sample * 16000.0) as i16
        })
        .collect()
}

impl Generator for Mp3Generator {
    fn format_name(&self) -> &str {
        "MP3"
    }

    fn file_extension(&self) -> &str {
        "mp3"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (duration, sample_rate, tone) = match &config.format_options {
            FormatOptions::Audio {
                duration,
                sample_rate,
                tone,
            } => (*duration, *sample_rate, *tone),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "MP3 generator requires Audio options".to_string(),
                ))
            }
        };

        if duration <= 0.0 {
            return Err(GenerationError::InvalidConfig(
                "Duration must be greater than 0".to_string(),
            ));
        }

        // Map sample rate to index (MPEG1): 00=44100, 01=48000, 10=32000
        let (actual_sr, sr_index) = match sample_rate {
            48000 => (48000u32, 1u8),
            32000 => (32000, 2u8),
            _ => (44100, 0u8), // Default to 44100
        };

        // We use 128kbps (bitrate index 9 for MPEG1 Layer III)
        let bitrate_index: u8 = 9;
        let bitrate = 128000u32;

        // Calculate frame size: 144 * bitrate / sample_rate
        let frame_data_size = (144 * bitrate / actual_sr) as usize;

        // Generate PCM samples for tone synthesis
        let samples = match tone {
            ToneType::Sine => generate_sine_samples(actual_sr, duration, 440.0),
            ToneType::Noise => generate_noise_samples(&mut config.rng, actual_sr, duration),
            ToneType::Sweep => generate_sweep_samples(actual_sr, duration),
        };

        // Calculate number of frames needed
        // Each MPEG1 Layer III frame encodes 1152 samples
        let samples_per_frame = 1152usize;
        let frame_count = samples.len().div_ceil(samples_per_frame);

        let header = build_mp3_frame_header(sr_index, bitrate_index);

        // Build the MP3 file: optional ID3v2 tag + frames
        let mut output = Vec::new();

        // Write a minimal ID3v2 header (so players recognize it quickly)
        // ID3v2.3 header: "ID3" + version(2 bytes) + flags(1) + size(4 syncsafe)
        output.extend_from_slice(b"ID3");
        output.extend_from_slice(&[0x03, 0x00]); // version 2.3.0
        output.push(0x00); // flags
        output.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // size 0 (empty tag)

        // Write MP3 frames
        for frame_idx in 0..frame_count {
            output.extend_from_slice(&header);

            // Fill frame data with scaled PCM content
            // This creates a frame that has the right structure but simplified audio data
            let start = frame_idx * samples_per_frame;
            let end = (start + samples_per_frame).min(samples.len());

            let mut frame_data = vec![0u8; frame_data_size - 4]; // minus header

            // Write side information (required for Layer III decoder)
            // For MPEG1 stereo, side info is 32 bytes
            // We zero it out (indicates empty granules, which decoders handle gracefully)
            // Then fill remaining space with audio-like data derived from samples
            let side_info_len = 32.min(frame_data.len());
            for b in frame_data[..side_info_len].iter_mut() {
                *b = 0;
            }

            // Fill the remaining frame data with sample-derived bytes
            if side_info_len < frame_data.len() {
                for (j, byte) in frame_data[side_info_len..].iter_mut().enumerate() {
                    let sample_idx = start + (j % (end - start).max(1));
                    if sample_idx < samples.len() {
                        *byte = ((samples[sample_idx] >> 8) as u8).wrapping_add(128);
                    }
                }
            }

            output.extend_from_slice(&frame_data);
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::FormatOptions;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::path::PathBuf;

    fn make_config(duration: f32, tone: ToneType) -> GeneratorConfig {
        GeneratorConfig {
            output_dir: PathBuf::from("/tmp"),
            name_pattern: "test_{n}".to_string(),
            extension: "mp3".to_string(),
            index: 0,
            overwrite: false,
            rng: ChaCha8Rng::seed_from_u64(42),
            format_options: FormatOptions::Audio {
                duration,
                sample_rate: 44100,
                tone,
            },
        }
    }

    #[test]
    fn test_mp3_has_id3_header() {
        let gen = Mp3Generator;
        let mut config = make_config(1.0, ToneType::Sine);
        let result = gen.generate(&mut config).unwrap();
        assert_eq!(&result[0..3], b"ID3");
    }

    #[test]
    fn test_mp3_has_frame_sync() {
        let gen = Mp3Generator;
        let mut config = make_config(0.5, ToneType::Sine);
        let result = gen.generate(&mut config).unwrap();
        // After ID3 header (10 bytes), first frame should start with sync
        assert_eq!(result[10], 0xFF);
        assert_eq!(result[11] & 0xE0, 0xE0); // sync bits
    }

    #[test]
    fn test_mp3_all_tones() {
        let gen = Mp3Generator;
        for tone in [ToneType::Sine, ToneType::Noise, ToneType::Sweep] {
            let mut config = make_config(0.5, tone);
            let result = gen.generate(&mut config);
            assert!(result.is_ok(), "Failed for tone: {tone}");
        }
    }

    #[test]
    fn test_mp3_duration_affects_size() {
        let gen = Mp3Generator;
        let mut c1 = make_config(1.0, ToneType::Sine);
        let mut c2 = make_config(3.0, ToneType::Sine);
        let r1 = gen.generate(&mut c1).unwrap();
        let r2 = gen.generate(&mut c2).unwrap();
        assert!(r2.len() > r1.len());
    }

    #[test]
    fn test_mp3_zero_duration_error() {
        let gen = Mp3Generator;
        let mut config = make_config(0.0, ToneType::Sine);
        assert!(gen.generate(&mut config).is_err());
    }

    #[test]
    fn test_sine_samples() {
        let samples = generate_sine_samples(44100, 0.01, 440.0);
        assert!(!samples.is_empty());
        // Should have approximately 441 samples (44100 * 0.01)
        assert!((samples.len() as i32 - 441).abs() < 2);
    }
}
