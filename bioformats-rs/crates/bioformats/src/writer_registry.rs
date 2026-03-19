use std::path::Path;

use bioformats_common::error::{BioFormatsError, Result};
use bioformats_common::metadata::ImageMetadata;
use bioformats_common::writer::FormatWriter;

/// Auto-detecting image writer. Choose an output format by file extension.
///
/// # Example
/// ```no_run
/// use bioformats::{ImageWriter, ImageMetadata, PixelType};
/// use std::path::Path;
///
/// let mut meta = ImageMetadata::default();
/// meta.size_x = 64; meta.size_y = 64;
/// meta.pixel_type = PixelType::Uint8;
/// meta.image_count = 1;
///
/// let data = vec![128u8; 64 * 64];
/// ImageWriter::save(Path::new("out.tif"), &meta, &[data]).unwrap();
/// ```
pub struct ImageWriter {
    inner: Box<dyn FormatWriter>,
}

fn writer_for(path: &Path) -> Option<Box<dyn FormatWriter>> {
    let writers: Vec<Box<dyn FormatWriter>> = vec![
        Box::new(bioformats_tiff::TiffWriter::new()),
        Box::new(bioformats_png::PngWriter::new()),
        Box::new(bioformats_jpeg::JpegWriter::new()),
        Box::new(bioformats_bmp::BmpWriter::new()),
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

    /// Lower-level: create a writer and stream planes manually.
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

    pub fn close(&mut self) -> Result<()> {
        self.inner.close()
    }
}
