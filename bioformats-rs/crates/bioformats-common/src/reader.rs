use std::path::Path;
use crate::error::Result;
use crate::metadata::ImageMetadata;
use crate::ome_metadata::OmeMetadata;

/// Core trait that every format reader must implement.
pub trait FormatReader: Send + Sync {
    fn is_this_type_by_name(&self, path: &Path) -> bool;
    fn is_this_type_by_bytes(&self, header: &[u8]) -> bool;
    fn set_id(&mut self, path: &Path) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn series_count(&self) -> usize;
    fn set_series(&mut self, series: usize) -> Result<()>;
    fn series(&self) -> usize;
    fn metadata(&self) -> &ImageMetadata;
    fn open_bytes(&mut self, plane_index: u32) -> Result<Vec<u8>>;
    fn open_bytes_region(&mut self, plane_index: u32, x: u32, y: u32, w: u32, h: u32) -> Result<Vec<u8>>;
    fn open_thumb_bytes(&mut self, plane_index: u32) -> Result<Vec<u8>>;
    fn resolution_count(&self) -> usize { 1 }
    fn set_resolution(&mut self, _level: usize) -> Result<()> { Ok(()) }
    fn resolution(&self) -> usize { 0 }
    /// Return structured OME metadata if this format carries it, otherwise `None`.
    ///
    /// Equivalent to Java Bio-Formats `reader.setMetadataStore(service.createOMEXMLMetadata())`.
    /// Populated for formats that embed OME-XML or equivalent rich metadata
    /// (CZI, OME-TIFF, OME-XML). Returns `None` for all other formats.
    fn ome_metadata(&self) -> Option<OmeMetadata> { None }
}
