use std::path::{Path, PathBuf};
use bioformats_common::error::{BioFormatsError, Result};
use bioformats_common::metadata::ImageMetadata;
use bioformats_common::pixel_type::PixelType;
use bioformats_common::writer::FormatWriter;

pub struct JpegWriter {
    path: Option<PathBuf>,
    meta: Option<ImageMetadata>,
    quality: u8,
}

impl JpegWriter {
    pub fn new() -> Self { JpegWriter { path: None, meta: None, quality: 90 } }
    pub fn with_quality(mut self, q: u8) -> Self { self.quality = q; self }
}

impl Default for JpegWriter { fn default() -> Self { Self::new() } }

impl FormatWriter for JpegWriter {
    fn is_this_type(&self, path: &Path) -> bool {
        path.extension().and_then(|e| e.to_str())
            .map(|e| matches!(e.to_ascii_lowercase().as_str(), "jpg" | "jpeg"))
            .unwrap_or(false)
    }

    fn set_metadata(&mut self, meta: &ImageMetadata) -> Result<()> {
        if meta.pixel_type != PixelType::Uint8 {
            return Err(BioFormatsError::UnsupportedFormat(
                "JPEG writer only supports Uint8".into()
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
            return Err(BioFormatsError::Format("JPEG writer supports only one plane".into()));
        }
        let meta = self.meta.as_ref().ok_or(BioFormatsError::NotInitialized)?;
        let path = self.path.as_ref().ok_or(BioFormatsError::NotInitialized)?;
        let (w, h) = (meta.size_x, meta.size_y);
        let spp = meta.size_c as usize;

        let img: image::DynamicImage = match spp {
            1 => image::GrayImage::from_raw(w, h, data.to_vec())
                    .map(image::DynamicImage::ImageLuma8)
                    .ok_or_else(|| BioFormatsError::InvalidData("bad data length".into()))?,
            3 => image::RgbImage::from_raw(w, h, data.to_vec())
                    .map(image::DynamicImage::ImageRgb8)
                    .ok_or_else(|| BioFormatsError::InvalidData("bad data length".into()))?,
            _ => return Err(BioFormatsError::UnsupportedFormat(
                    format!("JPEG writer: unsupported spp={}", spp))),
        };

        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
            std::fs::File::create(path).map_err(BioFormatsError::Io)?,
            self.quality,
        );
        img.write_with_encoder(encoder).map_err(|e| BioFormatsError::Format(e.to_string()))
    }

    fn can_do_stacks(&self) -> bool { false }
}
