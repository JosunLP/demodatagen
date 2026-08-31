//! WAV audio file generator.
//!
//! Produces genuinely valid, uncompressed PCM WAV files using the `hound`
//! crate, with synthesized tones (sine, white noise, or frequency sweep). Unlike
//! the MP3 stub, these files play correctly in any audio tool.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig, ToneType};
use crate::error::{GenResult, GenerationError};
use crate::formats::mp3::{generate_noise_samples, generate_sine_samples, generate_sweep_samples};
use std::io::Cursor;

/// Generator for WAV audio files.
pub struct WavGenerator;

impl Generator for WavGenerator {
    fn format_name(&self) -> &str {
        "WAV"
    }

    fn file_extension(&self) -> &str {
        "wav"
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
                    "WAV generator requires Audio options".to_string(),
                ));
            }
        };
        if duration <= 0.0 {
            return Err(GenerationError::InvalidConfig(
                "Duration must be greater than 0".to_string(),
            ));
        }
        if sample_rate == 0 {
            return Err(GenerationError::InvalidConfig(
                "Sample rate must be greater than 0".to_string(),
            ));
        }

        let samples = match tone {
            ToneType::Sine => generate_sine_samples(sample_rate, duration, 440.0),
            ToneType::Noise => generate_noise_samples(&mut config.rng, sample_rate, duration),
            ToneType::Sweep => generate_sweep_samples(sample_rate, duration),
        };

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = hound::WavWriter::new(&mut cursor, spec)
                .map_err(|e| GenerationError::Audio(e.to_string()))?;
            for sample in samples {
                writer
                    .write_sample(sample)
                    .map_err(|e| GenerationError::Audio(e.to_string()))?;
            }
            writer
                .finalize()
                .map_err(|e| GenerationError::Audio(e.to_string()))?;
        }
        Ok(cursor.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::audio_config;

    #[test]
    fn test_wav_valid_riff_header() {
        let mut config = audio_config(0.5, 44100, ToneType::Sine);
        let result = WavGenerator.generate(&mut config).unwrap();
        assert_eq!(&result[0..4], b"RIFF");
        assert_eq!(&result[8..12], b"WAVE");
    }

    #[test]
    fn test_wav_decodes_with_hound() {
        let mut config = audio_config(0.25, 44100, ToneType::Sweep);
        let result = WavGenerator.generate(&mut config).unwrap();
        let reader = hound::WavReader::new(Cursor::new(result)).unwrap();
        assert_eq!(reader.spec().sample_rate, 44100);
        assert_eq!(reader.spec().channels, 1);
        assert!(reader.len() > 0);
    }

    #[test]
    fn test_wav_all_tones() {
        for tone in [ToneType::Sine, ToneType::Noise, ToneType::Sweep] {
            let mut config = audio_config(0.2, 22050, tone);
            assert!(WavGenerator.generate(&mut config).is_ok());
        }
    }

    #[test]
    fn test_wav_zero_duration_error() {
        let mut config = audio_config(0.0, 44100, ToneType::Sine);
        assert!(WavGenerator.generate(&mut config).is_err());
    }
}
