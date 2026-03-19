use std::path::{Path, PathBuf};
use bioformats_common::error::{BioFormatsError, Result};
use bioformats_common::metadata::ImageMetadata;
use bioformats_common::pixel_type::PixelType;
use bioformats_common::writer::FormatWriter;

pub struct BmpWriter {
    path: Option<PathBuf>,
    meta: Option<ImageMetadata>,
}

impl BmpWriter {
    pub fn new() -> Self { BmpWriter { path: None, meta: None } }
}

impl Default for BmpWriter { fn default() -> Self { Self::new() } }

impl FormatWriter for BmpWriter {
    fn is_this_type(&self, path: &Path) -> bool {
        path.extension().and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("bmp"))
            .unwrap_or(false)
    }

    fn set_metadata(&mut self, meta: &ImageMetadata) -> Result<()> {
        if meta.pixel_type != PixelType::Uint8 {
            return Err(BioFormatsError::UnsupportedFormat(
                "BMP writer only supports Uint8".into()
            ));
        }
        self.meta = Some(meta.clone());
        Ok(())
    }

    fn set_id(&mut self, path: &Path) -> Result<()> {
        self.meta.as_ref().ok_or_else(|| BioFormatsError::Format("set_metadata first".into()))?;
        self.path = Some(path.to_path_buf());
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        self.path = None;
        self.meta = None;
        Ok(())
    }

    fn save_bytes(&mut self, plane_index: u32, data: &[u8]) -> Result<()> {
        if plane_index != 0 {
            return Err(BioFormatsError::Format("BMP writer supports only one plane".into()));
        }
        let meta = self.meta.as_ref().ok_or(BioFormatsError::NotInitialized)?;
        let path = self.path.as_ref().ok_or(BioFormatsError::NotInitialized)?;
        let (w, h) = (meta.size_x, meta.size_y);

        let img = image::RgbImage::from_raw(w, h, data.to_vec())
            .map(image::DynamicImage::ImageRgb8)
            .ok_or_else(|| BioFormatsError::InvalidData("bad data length".into()))?;

        img.save(path).map_err(|e| BioFormatsError::Format(e.to_string()))
    }

    fn can_do_stacks(&self) -> bool { false }
}
