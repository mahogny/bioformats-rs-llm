use std::path::{Path, PathBuf};
use bioformats_common::error::{BioFormatsError, Result};
use bioformats_common::metadata::ImageMetadata;
use bioformats_common::pixel_type::PixelType;
use bioformats_common::writer::FormatWriter;

pub struct PngWriter {
    path: Option<PathBuf>,
    meta: Option<ImageMetadata>,
}

impl PngWriter {
    pub fn new() -> Self { PngWriter { path: None, meta: None } }
}

impl Default for PngWriter { fn default() -> Self { Self::new() } }

impl FormatWriter for PngWriter {
    fn is_this_type(&self, path: &Path) -> bool {
        path.extension().and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("png"))
            .unwrap_or(false)
    }

    fn set_metadata(&mut self, meta: &ImageMetadata) -> Result<()> {
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
            return Err(BioFormatsError::Format("PNG writer supports only one plane".into()));
        }
        let meta = self.meta.as_ref().ok_or(BioFormatsError::NotInitialized)?;
        let path = self.path.as_ref().ok_or(BioFormatsError::NotInitialized)?;

        let (w, h) = (meta.size_x, meta.size_y);
        let spp = meta.size_c as usize;

        let img: image::DynamicImage = match (meta.pixel_type, spp) {
            (PixelType::Uint8, 1) => {
                image::GrayImage::from_raw(w, h, data.to_vec())
                    .map(image::DynamicImage::ImageLuma8)
                    .ok_or_else(|| BioFormatsError::InvalidData("bad data length".into()))?
            }
            (PixelType::Uint8, 3) => {
                image::RgbImage::from_raw(w, h, data.to_vec())
                    .map(image::DynamicImage::ImageRgb8)
                    .ok_or_else(|| BioFormatsError::InvalidData("bad data length".into()))?
            }
            (PixelType::Uint8, 4) => {
                image::RgbaImage::from_raw(w, h, data.to_vec())
                    .map(image::DynamicImage::ImageRgba8)
                    .ok_or_else(|| BioFormatsError::InvalidData("bad data length".into()))?
            }
            (PixelType::Uint16, 1) => {
                let pixels: Vec<u16> = data.chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                image::ImageBuffer::<image::Luma<u16>, _>::from_raw(w, h, pixels)
                    .map(image::DynamicImage::ImageLuma16)
                    .ok_or_else(|| BioFormatsError::InvalidData("bad data length".into()))?
            }
            (PixelType::Uint16, 3) => {
                let pixels: Vec<u16> = data.chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                image::ImageBuffer::<image::Rgb<u16>, _>::from_raw(w, h, pixels)
                    .map(image::DynamicImage::ImageRgb16)
                    .ok_or_else(|| BioFormatsError::InvalidData("bad data length".into()))?
            }
            _ => {
                return Err(BioFormatsError::UnsupportedFormat(format!(
                    "PNG writer: unsupported {:?} spp={}", meta.pixel_type, spp
                )));
            }
        };

        img.save(path).map_err(|e| BioFormatsError::Format(e.to_string()))
    }

    fn can_do_stacks(&self) -> bool { false }
}
