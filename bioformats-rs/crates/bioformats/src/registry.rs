use std::path::Path;

use bioformats_common::error::{BioFormatsError, Result};
use bioformats_common::metadata::ImageMetadata;
use bioformats_common::reader::FormatReader;
use bioformats_common::io::peek_header;

/// The top-level reader that auto-detects the file format and delegates to the
/// appropriate format-specific reader.
///
/// Mirrors `ImageReader` from the Java library.
pub struct ImageReader {
    inner: Box<dyn FormatReader>,
}

fn all_readers() -> Vec<Box<dyn FormatReader>> {
    vec![
        Box::new(bioformats_tiff::TiffReader::new()),
        Box::new(bioformats_png::PngReader::new()),
        Box::new(bioformats_jpeg::JpegReader::new()),
        Box::new(bioformats_bmp::BmpReader::new()),
    ]
}

impl ImageReader {
    /// Open the file at `path`, detect its format, parse metadata.
    pub fn open(path: &Path) -> Result<Self> {
        let header = peek_header(path, 512).unwrap_or_default();

        // 1. Magic bytes
        for mut r in all_readers() {
            if r.is_this_type_by_bytes(&header) {
                r.set_id(path)?;
                return Ok(ImageReader { inner: r });
            }
        }

        // 2. Extension fallback
        for mut r in all_readers() {
            if r.is_this_type_by_name(path) {
                r.set_id(path)?;
                return Ok(ImageReader { inner: r });
            }
        }

        Err(BioFormatsError::UnsupportedFormat(
            path.display().to_string(),
        ))
    }

    pub fn series_count(&self) -> usize { self.inner.series_count() }
    pub fn set_series(&mut self, series: usize) -> Result<()> { self.inner.set_series(series) }
    pub fn series(&self) -> usize { self.inner.series() }
    pub fn metadata(&self) -> &ImageMetadata { self.inner.metadata() }
    pub fn open_bytes(&mut self, plane_index: u32) -> Result<Vec<u8>> { self.inner.open_bytes(plane_index) }
    pub fn open_bytes_region(&mut self, plane_index: u32, x: u32, y: u32, w: u32, h: u32) -> Result<Vec<u8>> {
        self.inner.open_bytes_region(plane_index, x, y, w, h)
    }
    pub fn open_thumb_bytes(&mut self, plane_index: u32) -> Result<Vec<u8>> { self.inner.open_thumb_bytes(plane_index) }
    pub fn resolution_count(&self) -> usize { self.inner.resolution_count() }
    pub fn set_resolution(&mut self, level: usize) -> Result<()> { self.inner.set_resolution(level) }
    pub fn close(&mut self) -> Result<()> { self.inner.close() }
}
