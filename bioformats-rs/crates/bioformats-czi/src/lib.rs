//! Zeiss CZI reader — Phase 3 stub.
//!
//! CZI uses a segment-based container format with ZSTD/JPEG/JPEG-XR compressed tiles.
//! This stub detects CZI files but returns `UnsupportedFormat` until Phase 3 is implemented.

use std::path::{Path, PathBuf};
use bioformats_common::error::{BioFormatsError, Result};
use bioformats_common::metadata::ImageMetadata;
use bioformats_common::reader::FormatReader;

pub struct CziReader {
    _path: Option<PathBuf>,
}

impl CziReader {
    pub fn new() -> Self {
        CziReader { _path: None }
    }
}

impl Default for CziReader {
    fn default() -> Self { Self::new() }
}

impl FormatReader for CziReader {
    fn is_this_type_by_name(&self, path: &Path) -> bool {
        path.extension().and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("czi"))
            .unwrap_or(false)
    }

    fn is_this_type_by_bytes(&self, header: &[u8]) -> bool {
        // CZI magic: "ZISRAWFILE"
        header.starts_with(b"ZISRAWFILE")
    }

    fn set_id(&mut self, path: &Path) -> Result<()> {
        Err(BioFormatsError::UnsupportedFormat(
            format!("CZI support is planned for Phase 3: {}", path.display())
        ))
    }

    fn close(&mut self) -> Result<()> { Ok(()) }
    fn series_count(&self) -> usize { 0 }
    fn set_series(&mut self, s: usize) -> Result<()> { Err(BioFormatsError::SeriesOutOfRange(s)) }
    fn series(&self) -> usize { 0 }

    fn metadata(&self) -> &ImageMetadata {
        panic!("CziReader not initialized")
    }

    fn open_bytes(&mut self, _: u32) -> Result<Vec<u8>> {
        Err(BioFormatsError::NotInitialized)
    }
    fn open_bytes_region(&mut self, _: u32, _: u32, _: u32, _: u32, _: u32) -> Result<Vec<u8>> {
        Err(BioFormatsError::NotInitialized)
    }
    fn open_thumb_bytes(&mut self, _: u32) -> Result<Vec<u8>> {
        Err(BioFormatsError::NotInitialized)
    }
}
