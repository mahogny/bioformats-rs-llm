use bioformats::{ImageMetadata, ImageReader, ImageWriter, PixelType};
use std::path::Path;

fn tmp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("bioformats_fmt_{}", name))
}

fn round_trip(path: &Path, meta: &ImageMetadata, plane: Vec<u8>) -> Vec<u8> {
    ImageWriter::save(path, meta, &[plane]).expect("write failed");
    let mut r = ImageReader::open(path).expect("read failed");
    r.open_bytes(0).expect("open_bytes failed")
}

// ---- ICS -------------------------------------------------------------------

#[test]
fn ics_round_trip_gray8() {
    let mut meta = ImageMetadata::default();
    meta.size_x = 8; meta.size_y = 8;
    meta.pixel_type = PixelType::Uint8;
    meta.image_count = 1;

    let data: Vec<u8> = (0..64).collect();
    let rb = round_trip(&tmp("gray8.ics"), &meta, data.clone());
    assert_eq!(rb, data);
}

#[test]
fn ics_round_trip_gray16() {
    let mut meta = ImageMetadata::default();
    meta.size_x = 4; meta.size_y = 4;
    meta.pixel_type = PixelType::Uint16;
    meta.bits_per_pixel = 16;
    meta.image_count = 1;

    let data: Vec<u8> = (0u16..16).flat_map(|v| v.to_le_bytes()).collect();
    let rb = round_trip(&tmp("gray16.ics"), &meta, data.clone());
    assert_eq!(rb, data);
}

// ---- MRC -------------------------------------------------------------------

#[test]
fn mrc_round_trip_gray8() {
    let mut meta = ImageMetadata::default();
    meta.size_x = 8; meta.size_y = 8;
    meta.pixel_type = PixelType::Uint8;
    meta.image_count = 1;

    let data: Vec<u8> = (0..64).collect();
    ImageWriter::save(&tmp("test.mrc"), &meta, &[data.clone()]).unwrap();
    let mut r = ImageReader::open(&tmp("test.mrc")).unwrap();
    let rb = r.open_bytes(0).unwrap();
    // MRC flips rows; after double-flip (write+read) data should be identical
    assert_eq!(rb, data);
}

#[test]
fn mrc_round_trip_float32() {
    let mut meta = ImageMetadata::default();
    meta.size_x = 4; meta.size_y = 4;
    meta.pixel_type = PixelType::Float32;
    meta.bits_per_pixel = 32;
    meta.image_count = 1;

    let data: Vec<u8> = (0u32..16).flat_map(|v| (v as f32).to_le_bytes()).collect();
    ImageWriter::save(&tmp("float.mrc"), &meta, &[data.clone()]).unwrap();
    let mut r = ImageReader::open(&tmp("float.mrc")).unwrap();
    let rb = r.open_bytes(0).unwrap();
    assert_eq!(rb, data);
}

// ---- FITS ------------------------------------------------------------------

#[test]
fn fits_round_trip_gray8() {
    let mut meta = ImageMetadata::default();
    meta.size_x = 8; meta.size_y = 8;
    meta.pixel_type = PixelType::Uint8;
    meta.image_count = 1;

    let data: Vec<u8> = (0..64).collect();
    ImageWriter::save(&tmp("test.fits"), &meta, &[data.clone()]).unwrap();
    let mut r = ImageReader::open(&tmp("test.fits")).unwrap();
    let rb = r.open_bytes(0).unwrap();
    assert_eq!(rb, data);
}

#[test]
fn fits_multi_plane() {
    let mut meta = ImageMetadata::default();
    meta.size_x = 4; meta.size_y = 4;
    meta.pixel_type = PixelType::Uint8;
    meta.size_z = 3;
    meta.image_count = 3;

    let planes: Vec<Vec<u8>> = (0u8..3).map(|p| vec![p * 50; 16]).collect();
    ImageWriter::save(&tmp("stack.fits"), &meta, &planes).unwrap();
    let mut r = ImageReader::open(&tmp("stack.fits")).unwrap();
    assert_eq!(r.metadata().image_count, 3);
    for p in 0u8..3 {
        let plane = r.open_bytes(p as u32).unwrap();
        assert!(plane.iter().all(|&b| b == p * 50));
    }
}

// ---- NRRD ------------------------------------------------------------------

#[test]
fn nrrd_round_trip_gray8() {
    let mut meta = ImageMetadata::default();
    meta.size_x = 8; meta.size_y = 8;
    meta.pixel_type = PixelType::Uint8;
    meta.image_count = 1;

    let data: Vec<u8> = (0..64).collect();
    let rb = round_trip(&tmp("test.nrrd"), &meta, data.clone());
    assert_eq!(rb, data);
}

#[test]
fn nrrd_round_trip_float32() {
    let mut meta = ImageMetadata::default();
    meta.size_x = 4; meta.size_y = 4;
    meta.pixel_type = PixelType::Float32;
    meta.bits_per_pixel = 32;
    meta.image_count = 1;

    let data: Vec<u8> = (0u32..16).flat_map(|v| (v as f32).to_le_bytes()).collect();
    let rb = round_trip(&tmp("float.nrrd"), &meta, data.clone());
    assert_eq!(rb, data);
}

// ---- MetaImage (MHA) -------------------------------------------------------

#[test]
fn metaimage_mha_round_trip() {
    let mut meta = ImageMetadata::default();
    meta.size_x = 8; meta.size_y = 8;
    meta.pixel_type = PixelType::Uint8;
    meta.image_count = 1;

    let data: Vec<u8> = (0..64).collect();
    let rb = round_trip(&tmp("test.mha"), &meta, data.clone());
    assert_eq!(rb, data);
}

#[test]
fn metaimage_mhd_round_trip() {
    let mut meta = ImageMetadata::default();
    meta.size_x = 8; meta.size_y = 8;
    meta.pixel_type = PixelType::Uint8;
    meta.image_count = 1;

    let data: Vec<u8> = (0..64).collect();
    let rb = round_trip(&tmp("test.mhd"), &meta, data.clone());
    assert_eq!(rb, data);
}

// ---- raster formats --------------------------------------------------------

#[test]
fn tga_round_trip() {
    let mut meta = ImageMetadata::default();
    meta.size_x = 8; meta.size_y = 8;
    meta.pixel_type = PixelType::Uint8;
    meta.size_c = 3;
    meta.is_rgb = true;
    meta.image_count = 1;

    let data: Vec<u8> = (0u8..192).collect(); // 8*8*3
    let rb = round_trip(&tmp("test.tga"), &meta, data.clone());
    assert_eq!(rb, data);
}
