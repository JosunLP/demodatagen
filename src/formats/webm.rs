/// WebM video file generator.
///
/// Produces a valid WebM (Matroska subset) container with a minimal
/// VP8 video track. The EBML structure is built manually to ensure
/// proper format compliance.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::error::{GenResult, GenerationError};
use rand::Rng;

/// Generator for WebM video files.
pub struct WebmGenerator;

/// Writes an EBML element ID.
fn write_ebml_id(buf: &mut Vec<u8>, id: u32) {
    if id <= 0x7F {
        buf.push(id as u8);
    } else if id <= 0x3FFF {
        buf.extend_from_slice(&(id as u16).to_be_bytes());
    } else if id <= 0x1FFFFF {
        let bytes = id.to_be_bytes();
        buf.extend_from_slice(&bytes[1..4]);
    } else {
        buf.extend_from_slice(&id.to_be_bytes());
    }
}

/// Writes an EBML variable-length size.
fn write_ebml_size(buf: &mut Vec<u8>, size: u64) {
    if size < 0x7F {
        buf.push((size | 0x80) as u8);
    } else if size < 0x3FFF {
        let val = size | 0x4000;
        buf.extend_from_slice(&(val as u16).to_be_bytes());
    } else if size < 0x1FFFFF {
        let val = size | 0x200000;
        let bytes = (val as u32).to_be_bytes();
        buf.extend_from_slice(&bytes[1..4]);
    } else {
        let val = size | 0x10000000;
        buf.extend_from_slice(&(val as u32).to_be_bytes());
    }
}

/// Writes an unsigned integer element.
fn write_uint_element(buf: &mut Vec<u8>, id: u32, value: u64) {
    write_ebml_id(buf, id);
    if value <= 0xFF {
        write_ebml_size(buf, 1);
        buf.push(value as u8);
    } else if value <= 0xFFFF {
        write_ebml_size(buf, 2);
        buf.extend_from_slice(&(value as u16).to_be_bytes());
    } else if value <= 0xFFFFFFFF {
        write_ebml_size(buf, 4);
        buf.extend_from_slice(&(value as u32).to_be_bytes());
    } else {
        write_ebml_size(buf, 8);
        buf.extend_from_slice(&value.to_be_bytes());
    }
}

/// Writes a float element (always 8 bytes for f64).
fn write_float_element(buf: &mut Vec<u8>, id: u32, value: f64) {
    write_ebml_id(buf, id);
    write_ebml_size(buf, 8);
    buf.extend_from_slice(&value.to_be_bytes());
}

/// Writes a string element.
fn write_string_element(buf: &mut Vec<u8>, id: u32, value: &str) {
    write_ebml_id(buf, id);
    write_ebml_size(buf, value.len() as u64);
    buf.extend_from_slice(value.as_bytes());
}

/// Writes a binary element.
fn write_binary_element(buf: &mut Vec<u8>, id: u32, data: &[u8]) {
    write_ebml_id(buf, id);
    write_ebml_size(buf, data.len() as u64);
    buf.extend_from_slice(data);
}

/// Writes a master element (container).
fn write_master_element(buf: &mut Vec<u8>, id: u32, children: &[u8]) {
    write_ebml_id(buf, id);
    write_ebml_size(buf, children.len() as u64);
    buf.extend_from_slice(children);
}

impl Generator for WebmGenerator {
    fn format_name(&self) -> &str {
        "WebM"
    }

    fn file_extension(&self) -> &str {
        "webm"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (duration, width, height, fps) = match &config.format_options {
            FormatOptions::Video {
                duration,
                width,
                height,
                fps,
            } => (*duration, *width, *height, *fps),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "WebM generator requires Video options".to_string(),
                ))
            }
        };

        if duration <= 0.0 || width == 0 || height == 0 || fps == 0 {
            return Err(GenerationError::InvalidConfig(
                "Duration, width, height, and FPS must all be greater than 0".to_string(),
            ));
        }

        let total_frames = (duration * fps as f32) as u32;
        let frame_duration_ms = 1000.0 / fps as f64;
        let duration_ms = duration as f64 * 1000.0;

        let mut output = Vec::new();

        // EBML Header
        let mut ebml_header = Vec::new();
        write_uint_element(&mut ebml_header, 0x4286, 1); // EBMLVersion
        write_uint_element(&mut ebml_header, 0x42F7, 1); // EBMLReadVersion
        write_uint_element(&mut ebml_header, 0x42F2, 4); // EBMLMaxIDLength
        write_uint_element(&mut ebml_header, 0x42F3, 8); // EBMLMaxSizeLength
        write_string_element(&mut ebml_header, 0x4282, "webm"); // DocType
        write_uint_element(&mut ebml_header, 0x4287, 4); // DocTypeVersion
        write_uint_element(&mut ebml_header, 0x4285, 2); // DocTypeReadVersion
        write_master_element(&mut output, 0x1A45DFA3, &ebml_header);

        // Segment
        let mut segment_data = Vec::new();

        // Segment Info
        let mut info = Vec::new();
        write_uint_element(&mut info, 0x2AD7B1, 1000000); // TimestampScale (1ms)
        write_string_element(&mut info, 0x4D80, "demodatagen"); // MuxingApp
        write_string_element(&mut info, 0x5741, "demodatagen"); // WritingApp
        write_float_element(&mut info, 0x4489, duration_ms); // Duration
        write_master_element(&mut segment_data, 0x1549A966, &info);

        // Tracks
        let mut tracks = Vec::new();
        let mut track_entry = Vec::new();
        write_uint_element(&mut track_entry, 0xD7, 1); // TrackNumber
        write_uint_element(&mut track_entry, 0x73C5, 1); // TrackUID
        write_uint_element(&mut track_entry, 0x83, 1); // TrackType (video)
        write_string_element(&mut track_entry, 0x86, "V_VP8"); // CodecID

        // Video settings
        let mut video = Vec::new();
        write_uint_element(&mut video, 0xB0, width as u64); // PixelWidth
        write_uint_element(&mut video, 0xBA, height as u64); // PixelHeight
        write_master_element(&mut track_entry, 0xE0, &video);

        write_uint_element(
            &mut track_entry,
            0x23E383,
            (frame_duration_ms * 1000000.0) as u64,
        ); // DefaultDuration

        write_master_element(&mut tracks, 0xAE, &track_entry);
        write_master_element(&mut segment_data, 0x1654AE6B, &tracks);

        // Cluster with frames
        let mut cluster = Vec::new();
        write_uint_element(&mut cluster, 0xE7, 0); // Timestamp (cluster starts at 0)

        for i in 0..total_frames {
            let timestamp = (i as f64 * frame_duration_ms) as i16;

            // Minimal VP8 keyframe
            // VP8 frame header: 3 bytes for frame tag + 7 bytes for keyframe header
            let mut frame_data = Vec::new();

            // Frame tag (3 bytes): keyframe, version, show_frame, partition_size
            // Bit 0: keyframe (0 = key)
            // Bits 1-2: version
            // Bit 3: show_frame
            // Bits 4-23: first_partition_size
            let partition_size: u32 = 0;
            let frame_tag = (partition_size << 5) | 0x10; // show_frame=1, version=0, keyframe=0
            frame_data.push((frame_tag & 0xFF) as u8);
            frame_data.push(((frame_tag >> 8) & 0xFF) as u8);
            frame_data.push(((frame_tag >> 16) & 0xFF) as u8);

            // Keyframe header (7 bytes)
            frame_data.extend_from_slice(&[0x9D, 0x01, 0x2A]); // start code
            frame_data.extend_from_slice(&(width as u16).to_le_bytes()); // width
            frame_data.extend_from_slice(&(height as u16).to_le_bytes()); // height

            // Add some random payload
            let payload_size = config.rng.gen_range(4..16);
            for _ in 0..payload_size {
                frame_data.push(config.rng.gen());
            }

            // SimpleBlock
            let mut block = Vec::new();
            block.push(0x81); // Track number 1 (EBML coded)
            block.extend_from_slice(&timestamp.to_be_bytes()); // Timecode relative to cluster
            let flags: u8 = if i == 0 { 0x80 } else { 0x00 }; // keyframe flag
            block.push(flags);
            block.extend_from_slice(&frame_data);

            write_binary_element(&mut cluster, 0xA3, &block); // SimpleBlock
        }

        write_master_element(&mut segment_data, 0x1F43B675, &cluster);

        write_master_element(&mut output, 0x18538067, &segment_data);

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::video_config as make_config;

    #[test]
    fn test_webm_valid_ebml_header() {
        let gen = WebmGenerator;
        let mut config = make_config(0.5, 160, 120, 10);
        let result = gen.generate(&mut config).unwrap();
        // EBML header starts with 0x1A45DFA3
        assert_eq!(result[0], 0x1A);
        assert_eq!(result[1], 0x45);
        assert_eq!(result[2], 0xDF);
        assert_eq!(result[3], 0xA3);
    }

    #[test]
    fn test_webm_contains_webm_doctype() {
        let gen = WebmGenerator;
        let mut config = make_config(0.5, 160, 120, 10);
        let result = gen.generate(&mut config).unwrap();
        // Should contain "webm" string
        let has_webm = result.windows(4).any(|w| w == b"webm");
        assert!(has_webm, "WebM file must contain 'webm' doctype");
    }

    #[test]
    fn test_webm_zero_params_error() {
        let gen = WebmGenerator;
        let mut config = make_config(0.0, 160, 120, 10);
        assert!(gen.generate(&mut config).is_err());
    }
}
