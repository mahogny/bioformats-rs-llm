use std::path::Path;

use bioformats_common::error::{BioFormatsError, Result};
use bioformats_common::metadata::ImageMetadata;
use bioformats_common::writer::FormatWriter;

/// Auto-detecting image writer. Choose an output format by file extension.
pub struct ImageWriter {
    inner: Box<dyn FormatWriter>,
}

fn writer_for(path: &Path) -> Option<Box<dyn FormatWriter>> {
    let writers: Vec<Box<dyn FormatWriter>> = vec![
        Box::new(bioformats_tiff::TiffWriter::new()),
        Box::new(crate::formats::png::PngWriter::new()),
        Box::new(crate::formats::jpeg::JpegWriter::new()),
        Box::new(crate::formats::bmp::BmpWriter::new()),
        Box::new(crate::formats::raster::TgaWriter::new()),
        Box::new(crate::formats::ics::IcsWriter::new()),
        Box::new(crate::formats::mrc::MrcWriter::new()),
        Box::new(crate::formats::fits::FitsWriter::new()),
        Box::new(crate::formats::nrrd::NrrdWriter::new()),
        Box::new(crate::formats::metaimage::MetaImageWriter::new()),
    ];
    writers.into_iter().find(|w| w.is_this_type(path))
}

impl ImageWriter {
    /// Convenience: write all planes in one call.
    pub fn save(path: &Path, meta: &ImageMetadata, planes: &[Vec<u8>]) -> Result<()> {
        let mut w = writer_for(path).ok_or_else(|| {
            BioFormatsError::UnsupportedFormat(path.display().to_string())
        })?;
        w.set_metadata(meta)?;
        w.set_id(path)?;
        for (i, plane) in planes.iter().enumerate() {
            w.save_bytes(i as u32, plane)?;
        }
        w.close()
    }

    /// Lower-level: stream planes manually.
    pub fn open(path: &Path, meta: &ImageMetadata) -> Result<Self> {
        let mut w = writer_for(path).ok_or_else(|| {
            BioFormatsError::UnsupportedFormat(path.display().to_string())
        })?;
        w.set_metadata(meta)?;
        w.set_id(path)?;
        Ok(ImageWriter { inner: w })
    }

    pub fn save_bytes(&mut self, plane_index: u32, data: &[u8]) -> Result<()> {
        self.inner.save_bytes(plane_index, data)
    }

    pub fn close(&mut self) -> Result<()> { self.inner.close() }
}
