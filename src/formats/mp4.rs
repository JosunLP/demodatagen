/// MP4 video file generator.
///
/// Produces a valid MP4 (ISO Base Media File Format) container with
/// a minimal video track. The file uses uncompressed video frames stored
/// in an `mdat` box, referenced by a proper `moov` box structure.
///
/// The generated file is recognized by media players as a valid MP4,
/// though the video content is simple colored frames.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::error::{GenResult, GenerationError};
use rand::RngExt;

/// Generator for MP4 video files.
pub struct Mp4Generator;

/// Writes a big-endian u32 to a vector.
fn write_u32(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&val.to_be_bytes());
}

/// Writes a big-endian u16 to a vector.
fn write_u16(buf: &mut Vec<u8>, val: u16) {
    buf.extend_from_slice(&val.to_be_bytes());
}

/// Builds an MP4 box (atom) with the given type and data.
fn build_box(box_type: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let size = (data.len() + 8) as u32;
    let mut result = Vec::with_capacity(size as usize);
    write_u32(&mut result, size);
    result.extend_from_slice(box_type);
    result.extend_from_slice(data);
    result
}

/// Builds a full-box (version + flags) with the given data.
fn build_full_box(box_type: &[u8; 4], version: u8, flags: u32, data: &[u8]) -> Vec<u8> {
    let mut inner = Vec::new();
    inner.push(version);
    let flag_bytes = flags.to_be_bytes();
    inner.extend_from_slice(&flag_bytes[1..4]);
    inner.extend_from_slice(data);
    build_box(box_type, &inner)
}

impl Generator for Mp4Generator {
    fn format_name(&self) -> &str {
        "MP4"
    }

    fn file_extension(&self) -> &str {
        "mp4"
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
                    "MP4 generator requires Video options".to_string(),
                ))
            }
        };

        if duration <= 0.0 || width == 0 || height == 0 || fps == 0 {
            return Err(GenerationError::InvalidConfig(
                "Duration, width, height, and FPS must all be greater than 0".to_string(),
            ));
        }

        let total_frames = (duration * fps as f32) as u32;
        let timescale = 1000u32;
        let duration_ms = (duration * 1000.0) as u32;
        let frame_duration_ts = timescale / fps;

        // Generate simple raw frame data (uncompressed RGB data, very small)
        // We use small 2x2 pixel I-frames to keep file size manageable
        let frame_w = 2u32;
        let frame_h = 2u32;
        let mut frames_data = Vec::new();
        let mut sample_sizes: Vec<u32> = Vec::new();

        for _ in 0..total_frames {
            // Simple YUV 4:2:0 frame (for minimal valid video)
            // Y plane: frame_w * frame_h bytes
            // U plane: (frame_w/2) * (frame_h/2) bytes
            // V plane: (frame_w/2) * (frame_h/2) bytes
            let y_val: u8 = config.rng.random_range(16..235);
            let u_val: u8 = config.rng.random_range(16..240);
            let v_val: u8 = config.rng.random_range(16..240);

            let y_size = (frame_w * frame_h) as usize;
            let uv_size = ((frame_w / 2) * (frame_h / 2)) as usize;
            let frame_size = y_size + 2 * uv_size;

            let mut frame = vec![y_val; y_size];
            frame.extend(vec![u_val; uv_size]);
            frame.extend(vec![v_val; uv_size]);

            sample_sizes.push(frame_size as u32);
            frames_data.extend_from_slice(&frame);
        }

        // Build the MP4 structure
        let mut output = Vec::new();

        // ftyp box
        let mut ftyp_data = Vec::new();
        ftyp_data.extend_from_slice(b"isom"); // major brand
        write_u32(&mut ftyp_data, 0x200); // minor version
        ftyp_data.extend_from_slice(b"isomiso2mp41"); // compatible brands
        output.extend_from_slice(&build_box(b"ftyp", &ftyp_data));

        // mdat box (media data)
        let mdat = build_box(b"mdat", &frames_data);
        let mdat_offset = output.len() as u32;
        output.extend_from_slice(&mdat);

        // Build moov box
        let moov = build_moov(
            width as u16,
            height as u16,
            frame_w as u16,
            frame_h as u16,
            timescale,
            duration_ms,
            frame_duration_ts,
            total_frames,
            &sample_sizes,
            mdat_offset + 8, // offset to first sample in mdat (skip mdat header)
        );
        output.extend_from_slice(&moov);

        Ok(output)
    }
}

/// Builds the `moov` box containing movie metadata and track info.
#[allow(clippy::too_many_arguments)]
fn build_moov(
    display_w: u16,
    display_h: u16,
    coded_w: u16,
    coded_h: u16,
    timescale: u32,
    duration_ms: u32,
    frame_duration_ts: u32,
    total_frames: u32,
    sample_sizes: &[u32],
    data_offset: u32,
) -> Vec<u8> {
    let mut moov_data = Vec::new();

    // mvhd (movie header)
    let mut mvhd = Vec::new();
    write_u32(&mut mvhd, 0); // creation time
    write_u32(&mut mvhd, 0); // modification time
    write_u32(&mut mvhd, timescale); // timescale
    write_u32(&mut mvhd, duration_ms); // duration
    write_u32(&mut mvhd, 0x00010000); // rate (1.0 fixed point)
    write_u16(&mut mvhd, 0x0100); // volume (1.0 fixed point)
    mvhd.extend_from_slice(&[0u8; 10]); // reserved
    let identity_matrix: [u8; 36] = [
        // Matrix (identity, 36 bytes)
        0x00, 0x01, 0x00, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x00, 0x01, 0x00, 0x00, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x00, 0x00, 0x00,
    ];
    mvhd.extend_from_slice(&identity_matrix);
    mvhd.extend_from_slice(&[0u8; 24]); // pre-defined
    write_u32(&mut mvhd, 2); // next track ID
    moov_data.extend_from_slice(&build_full_box(b"mvhd", 0, 0, &mvhd));

    // trak box
    let trak = build_trak(
        display_w,
        display_h,
        coded_w,
        coded_h,
        timescale,
        duration_ms,
        frame_duration_ts,
        total_frames,
        sample_sizes,
        data_offset,
    );
    moov_data.extend_from_slice(&trak);

    build_box(b"moov", &moov_data)
}

/// Builds a `trak` box for the video track.
#[allow(clippy::too_many_arguments)]
fn build_trak(
    display_w: u16,
    display_h: u16,
    coded_w: u16,
    coded_h: u16,
    timescale: u32,
    duration_ms: u32,
    frame_duration_ts: u32,
    total_frames: u32,
    sample_sizes: &[u32],
    data_offset: u32,
) -> Vec<u8> {
    let mut trak_data = Vec::new();

    // tkhd (track header)
    let mut tkhd = Vec::new();
    write_u32(&mut tkhd, 0); // creation time
    write_u32(&mut tkhd, 0); // modification time
    write_u32(&mut tkhd, 1); // track ID
    write_u32(&mut tkhd, 0); // reserved
    write_u32(&mut tkhd, duration_ms); // duration
    tkhd.extend_from_slice(&[0u8; 8]); // reserved
    write_u16(&mut tkhd, 0); // layer
    write_u16(&mut tkhd, 0); // alternate group
    write_u16(&mut tkhd, 0); // volume (0 for video)
    write_u16(&mut tkhd, 0); // reserved
                             // Matrix (identity)
    let identity_matrix: [u8; 36] = [
        0x00, 0x01, 0x00, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x00, 0x01, 0x00, 0x00, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x00, 0x00, 0x00,
    ];
    tkhd.extend_from_slice(&identity_matrix);
    write_u16(&mut tkhd, display_w);
    write_u16(&mut tkhd, 0); // width decimal
    write_u16(&mut tkhd, display_h);
    write_u16(&mut tkhd, 0); // height decimal
    trak_data.extend_from_slice(&build_full_box(b"tkhd", 0, 3, &tkhd));

    // mdia box
    let mdia = build_mdia(
        coded_w,
        coded_h,
        timescale,
        duration_ms,
        frame_duration_ts,
        total_frames,
        sample_sizes,
        data_offset,
    );
    trak_data.extend_from_slice(&mdia);

    build_box(b"trak", &trak_data)
}

/// Builds the `mdia` box containing media information.
#[allow(clippy::too_many_arguments)]
fn build_mdia(
    coded_w: u16,
    coded_h: u16,
    timescale: u32,
    duration_ms: u32,
    frame_duration_ts: u32,
    total_frames: u32,
    sample_sizes: &[u32],
    data_offset: u32,
) -> Vec<u8> {
    let mut mdia_data = Vec::new();

    // mdhd (media header)
    let mut mdhd = Vec::new();
    write_u32(&mut mdhd, 0); // creation time
    write_u32(&mut mdhd, 0); // modification time
    write_u32(&mut mdhd, timescale); // timescale
    write_u32(&mut mdhd, duration_ms); // duration
    write_u16(&mut mdhd, 0x55C4); // language (undetermined)
    write_u16(&mut mdhd, 0); // pre-defined
    mdia_data.extend_from_slice(&build_full_box(b"mdhd", 0, 0, &mdhd));

    // hdlr (handler)
    let mut hdlr = Vec::new();
    write_u32(&mut hdlr, 0); // pre-defined
    hdlr.extend_from_slice(b"vide"); // handler type
    hdlr.extend_from_slice(&[0u8; 12]); // reserved
    hdlr.extend_from_slice(b"VideoHandler\0"); // name
    mdia_data.extend_from_slice(&build_full_box(b"hdlr", 0, 0, &hdlr));

    // minf box
    let minf = build_minf(
        coded_w,
        coded_h,
        timescale,
        frame_duration_ts,
        total_frames,
        sample_sizes,
        data_offset,
    );
    mdia_data.extend_from_slice(&minf);

    build_box(b"mdia", &mdia_data)
}

/// Builds the `minf` box containing media information.
fn build_minf(
    coded_w: u16,
    coded_h: u16,
    _timescale: u32,
    frame_duration_ts: u32,
    total_frames: u32,
    sample_sizes: &[u32],
    data_offset: u32,
) -> Vec<u8> {
    let mut minf_data = Vec::new();

    // vmhd (video media header)
    let vmhd = vec![0u8; 8]; // graphics mode + opcolor
    minf_data.extend_from_slice(&build_full_box(b"vmhd", 0, 1, &vmhd));

    // dinf box
    let mut dref_data = Vec::new();
    write_u32(&mut dref_data, 1); // entry count
    dref_data.extend_from_slice(&build_full_box(b"url ", 0, 1, &[])); // self-contained
    let dinf = build_box(b"dinf", &build_full_box(b"dref", 0, 0, &dref_data));
    minf_data.extend_from_slice(&dinf);

    // stbl box
    let stbl = build_stbl(
        coded_w,
        coded_h,
        frame_duration_ts,
        total_frames,
        sample_sizes,
        data_offset,
    );
    minf_data.extend_from_slice(&stbl);

    build_box(b"minf", &minf_data)
}

/// Builds the `stbl` (sample table) box.
fn build_stbl(
    coded_w: u16,
    coded_h: u16,
    frame_duration_ts: u32,
    total_frames: u32,
    sample_sizes: &[u32],
    data_offset: u32,
) -> Vec<u8> {
    let mut stbl_data = Vec::new();

    // stsd (sample description)
    let mut stsd = Vec::new();
    write_u32(&mut stsd, 1); // entry count
                             // Visual sample entry (using 'raw ' codec for uncompressed)
    let mut visual_entry = Vec::new();
    visual_entry.extend_from_slice(&[0u8; 6]); // reserved
    write_u16(&mut visual_entry, 1); // data reference index
    write_u16(&mut visual_entry, 0); // pre-defined
    write_u16(&mut visual_entry, 0); // reserved
    visual_entry.extend_from_slice(&[0u8; 12]); // pre-defined
    write_u16(&mut visual_entry, coded_w); // width
    write_u16(&mut visual_entry, coded_h); // height
    write_u32(&mut visual_entry, 0x00480000); // horiz resolution 72dpi
    write_u32(&mut visual_entry, 0x00480000); // vert resolution 72dpi
    write_u32(&mut visual_entry, 0); // reserved
    write_u16(&mut visual_entry, 1); // frame count
    visual_entry.extend_from_slice(&[0u8; 32]); // compressor name
    write_u16(&mut visual_entry, 0x0018); // depth (24 bit)
    write_u16(&mut visual_entry, 0xFFFF); // pre-defined (-1)
    let visual_box = build_box(b"raw ", &visual_entry);
    stsd.extend_from_slice(&visual_box);
    stbl_data.extend_from_slice(&build_full_box(b"stsd", 0, 0, &stsd));

    // stts (time-to-sample)
    let mut stts = Vec::new();
    write_u32(&mut stts, 1); // entry count
    write_u32(&mut stts, total_frames); // sample count
    write_u32(&mut stts, frame_duration_ts); // sample delta
    stbl_data.extend_from_slice(&build_full_box(b"stts", 0, 0, &stts));

    // stsc (sample-to-chunk)
    let mut stsc = Vec::new();
    write_u32(&mut stsc, 1); // entry count
    write_u32(&mut stsc, 1); // first chunk
    write_u32(&mut stsc, total_frames); // samples per chunk
    write_u32(&mut stsc, 1); // sample description index
    stbl_data.extend_from_slice(&build_full_box(b"stsc", 0, 0, &stsc));

    // stsz (sample size)
    let mut stsz = Vec::new();
    write_u32(&mut stsz, 0); // sample size (0 = variable)
    write_u32(&mut stsz, total_frames); // sample count
    for &size in sample_sizes {
        write_u32(&mut stsz, size);
    }
    stbl_data.extend_from_slice(&build_full_box(b"stsz", 0, 0, &stsz));

    // stco (chunk offset)
    let mut stco = Vec::new();
    write_u32(&mut stco, 1); // entry count
    write_u32(&mut stco, data_offset); // offset to first chunk
    stbl_data.extend_from_slice(&build_full_box(b"stco", 0, 0, &stco));

    build_box(b"stbl", &stbl_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::video_config as make_config;

    #[test]
    fn test_mp4_valid_ftyp() {
        let gen = Mp4Generator;
        let mut config = make_config(1.0, 320, 240, 24);
        let result = gen.generate(&mut config).unwrap();
        // Check ftyp box
        assert_eq!(&result[4..8], b"ftyp");
        assert_eq!(&result[8..12], b"isom");
    }

    #[test]
    fn test_mp4_contains_moov() {
        let gen = Mp4Generator;
        let mut config = make_config(0.5, 160, 120, 10);
        let result = gen.generate(&mut config).unwrap();
        let data = &result;
        // Search for 'moov' box type
        let has_moov = data.windows(4).any(|w| w == b"moov");
        assert!(has_moov, "MP4 must contain moov box");
    }

    #[test]
    fn test_mp4_contains_mdat() {
        let gen = Mp4Generator;
        let mut config = make_config(0.5, 160, 120, 10);
        let result = gen.generate(&mut config).unwrap();
        let has_mdat = result.windows(4).any(|w| w == b"mdat");
        assert!(has_mdat, "MP4 must contain mdat box");
    }

    #[test]
    fn test_mp4_zero_dimension_error() {
        let gen = Mp4Generator;
        let mut config = make_config(1.0, 0, 240, 24);
        assert!(gen.generate(&mut config).is_err());
    }

    #[test]
    fn test_mp4_duration_affects_size() {
        let gen = Mp4Generator;
        let mut c1 = make_config(0.5, 160, 120, 10);
        let mut c2 = make_config(2.0, 160, 120, 10);
        let r1 = gen.generate(&mut c1).unwrap();
        let r2 = gen.generate(&mut c2).unwrap();
        assert!(r2.len() > r1.len());
    }
}
