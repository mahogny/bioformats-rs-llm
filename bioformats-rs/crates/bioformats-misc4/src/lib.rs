//! Placeholder readers for remaining obscure and proprietary formats.
//!
//! All readers are extension-only and return 512×512 uint8 placeholder metadata
//! with zeroed pixel data. Full decoding is not implemented.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bioformats_common::error::{BioFormatsError, Result};
use bioformats_common::metadata::{DimensionOrder, ImageMetadata};
use bioformats_common::pixel_type::PixelType;
use bioformats_common::reader::FormatReader;

// ---------------------------------------------------------------------------
// Shared placeholder metadata
// ---------------------------------------------------------------------------
fn placeholder_meta() -> ImageMetadata {
    ImageMetadata {
        size_x: 512,
        size_y: 512,
        size_z: 1,
        size_c: 1,
        size_t: 1,
        pixel_type: PixelType::Uint8,
        bits_per_pixel: 8,
        image_count: 1,
        dimension_order: DimensionOrder::XYZCT,
        is_rgb: false,
        is_interleaved: false,
        is_indexed: false,
        is_little_endian: true,
        resolution_count: 1,
        series_metadata: HashMap::new(),
        lookup_table: None,
    }
}

// ---------------------------------------------------------------------------
// Macro for extension-only placeholder readers
// ---------------------------------------------------------------------------
macro_rules! placeholder_reader {
    (
        $(#[$attr:meta])*
        pub struct $name:ident;
        extensions: [$($ext:literal),+];
        magic_bytes: false;
    ) => {
        $(#[$attr])*
        pub struct $name {
            path: Option<PathBuf>,
            meta: Option<ImageMetadata>,
        }

        impl $name {
            pub fn new() -> Self {
                $name { path: None, meta: None }
            }
        }

        impl Default for $name {
            fn default() -> Self { Self::new() }
        }

        impl FormatReader for $name {
            fn is_this_type_by_name(&self, path: &Path) -> bool {
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_ascii_lowercase());
                matches!(ext.as_deref(), $(Some($ext))|+)
            }

            fn is_this_type_by_bytes(&self, _header: &[u8]) -> bool { false }

            fn set_id(&mut self, path: &Path) -> Result<()> {
                self.path = Some(path.to_path_buf());
                self.meta = Some(placeholder_meta());
                Ok(())
            }

            fn close(&mut self) -> Result<()> {
                self.path = None;
                self.meta = None;
                Ok(())
            }

            fn series_count(&self) -> usize { 1 }

            fn set_series(&mut self, s: usize) -> Result<()> {
                if s != 0 { Err(BioFormatsError::SeriesOutOfRange(s)) } else { Ok(()) }
            }

            fn series(&self) -> usize { 0 }

            fn metadata(&self) -> &ImageMetadata {
                self.meta.as_ref().expect("set_id not called")
            }

            fn open_bytes(&mut self, plane_index: u32) -> Result<Vec<u8>> {
                let meta = self.meta.as_ref().ok_or(BioFormatsError::NotInitialized)?;
                if plane_index >= meta.image_count {
                    return Err(BioFormatsError::PlaneOutOfRange(plane_index));
                }
                Ok(vec![0u8; meta.size_x as usize * meta.size_y as usize])
            }

            fn open_bytes_region(&mut self, plane_index: u32, _x: u32, _y: u32, w: u32, h: u32) -> Result<Vec<u8>> {
                let meta = self.meta.as_ref().ok_or(BioFormatsError::NotInitialized)?;
                if plane_index >= meta.image_count {
                    return Err(BioFormatsError::PlaneOutOfRange(plane_index));
                }
                Ok(vec![0u8; w as usize * h as usize])
            }

            fn open_thumb_bytes(&mut self, plane_index: u32) -> Result<Vec<u8>> {
                let meta = self.meta.as_ref().ok_or(BioFormatsError::NotInitialized)?;
                let tw = meta.size_x.min(256);
                let th = meta.size_y.min(256);
                let tx = (meta.size_x - tw) / 2;
                let ty = (meta.size_y - th) / 2;
                self.open_bytes_region(plane_index, tx, ty, tw, th)
            }
        }
    };
}

// ---------------------------------------------------------------------------
// 1. Applied Precision APL
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Applied Precision format placeholder reader (`.apl`).
    pub struct AplReader;
    extensions: ["apl"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 2. ARF format
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// ARF format placeholder reader (`.arf`).
    pub struct ArfReader;
    extensions: ["arf"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 3. I2I format
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// I2I format placeholder reader (`.i2i`).
    pub struct I2iReader;
    extensions: ["i2i"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 4. JDCE format
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// JDCE format placeholder reader (`.jdce`).
    pub struct JdceReader;
    extensions: ["jdce"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 5. JPX (JPEG 2000 Part 2)
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// JPX (JPEG 2000 Part 2) format placeholder reader (`.jpx`).
    pub struct JpxReader;
    extensions: ["jpx"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 6. Capture Pro Image (PCI)
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Capture Pro Image format placeholder reader (`.pci`).
    pub struct PciReader;
    extensions: ["pci"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 7. PDS planetary format
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// PDS planetary format placeholder reader (`.pds`).
    pub struct PdsReader;
    extensions: ["pds"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 8. Hiscan HIS format
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Hiscan HIS format placeholder reader (`.his`).
    pub struct HisReader;
    extensions: ["his"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 9. HRDC GDF format
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// HRDC GDF format placeholder reader (`.gdf`).
    pub struct HrdgdfReader;
    extensions: ["gdf"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 10. Text/CSV image format
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Text/CSV image format placeholder reader (`.csv`).
    pub struct TextImageReader;
    extensions: ["csv"];
    magic_bytes: false;
}
