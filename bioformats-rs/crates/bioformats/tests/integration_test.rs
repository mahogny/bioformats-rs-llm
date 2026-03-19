use bioformats::ImageReader;
use std::path::Path;

#[test]
fn test_tiff_8x8_gray8() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/test_8x8_gray8.tif");
    let mut reader = ImageReader::open(&path).expect("open failed");

    assert_eq!(reader.series_count(), 1);

    let meta = reader.metadata();
    assert_eq!(meta.size_x, 8);
    assert_eq!(meta.size_y, 8);
    assert_eq!(meta.image_count, 1);
    assert_eq!(meta.pixel_type, bioformats::PixelType::Uint8);

    let plane = reader.open_bytes(0).expect("open_bytes failed");
    assert_eq!(plane.len(), 64);

    // Data should be ascending ramp 0..63
    for (i, &byte) in plane.iter().enumerate() {
        assert_eq!(byte, i as u8, "pixel {i} mismatch");
    }
}

#[test]
fn test_unknown_file_returns_error() {
    let path = Path::new("/tmp/nonexistent_bioformats_test.xyz");
    assert!(ImageReader::open(path).is_err());
}

#[test]
fn test_tiff_region() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/test_8x8_gray8.tif");
    let mut reader = ImageReader::open(&path).expect("open failed");

    // Read a 4x4 sub-region starting at (2, 2)
    let region = reader.open_bytes_region(0, 2, 2, 4, 4).expect("region failed");
    assert_eq!(region.len(), 16); // 4*4*1 byte

    // Row 2 of original starts at offset 16, pixels 16..23 = [16,17,18,19,20,21,22,23]
    // Starting at x=2 → bytes 18,19,20,21 for first row of region
    assert_eq!(region[0], 2 + 2 * 8); // row 2, col 2 = 18
}
