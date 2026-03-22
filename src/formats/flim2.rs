//! Additional FLIM, flow cytometry, and miscellaneous imaging format readers.
//!
//! Includes FlowSightReader with basic binary header inspection and many
//! extension-only placeholder readers.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::common::error::{BioFormatsError, Result};
use crate::common::metadata::{DimensionOrder, ImageMetadata};
use crate::common::pixel_type::PixelType;
use crate::common::reader::FormatReader;

// ---------------------------------------------------------------------------
// Shared helpers
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

fn placeholder_meta_u16_small() -> ImageMetadata {
    ImageMetadata {
        size_x: 64,
        size_y: 64,
        size_z: 1,
        size_c: 1,
        size_t: 1,
        pixel_type: PixelType::Uint16,
        bits_per_pixel: 16,
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
// Macros
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

macro_rules! placeholder_reader_u16_small {
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
                self.meta = Some(placeholder_meta_u16_small());
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
                Ok(vec![0u8; meta.size_x as usize * meta.size_y as usize * 2])
            }

            fn open_bytes_region(&mut self, plane_index: u32, _x: u32, _y: u32, w: u32, h: u32) -> Result<Vec<u8>> {
                let meta = self.meta.as_ref().ok_or(BioFormatsError::NotInitialized)?;
                if plane_index >= meta.image_count {
                    return Err(BioFormatsError::PlaneOutOfRange(plane_index));
                }
                Ok(vec![0u8; w as usize * h as usize * 2])
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
// 1. Amnis FlowSight (.cif)
// ---------------------------------------------------------------------------
/// Amnis FlowSight CIF format (`.cif`).
///
/// Returns a 64x64 uint16 placeholder; full decoding not implemented.
pub struct FlowSightReader {
    path: Option<PathBuf>,
    meta: Option<ImageMetadata>,
}

impl FlowSightReader {
    pub fn new() -> Self {
        FlowSightReader { path: None, meta: None }
    }
}

impl Default for FlowSightReader {
    fn default() -> Self { Self::new() }
}

impl FormatReader for FlowSightReader {
    fn is_this_type_by_name(&self, path: &Path) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref(),
            Some("cif")
        )
    }

    fn is_this_type_by_bytes(&self, _header: &[u8]) -> bool { false }

    fn set_id(&mut self, path: &Path) -> Result<()> {
        // Attempt to peek at first 4 bytes (informational only)
        let _magic: Option<[u8; 4]> = std::fs::File::open(path)
            .ok()
            .and_then(|mut f| {
                let mut buf = [0u8; 4];
                f.read_exact(&mut buf).ok().map(|_| buf)
            });
        self.path = Some(path.to_path_buf());
        self.meta = Some(placeholder_meta_u16_small());
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
        Ok(vec![0u8; meta.size_x as usize * meta.size_y as usize * 2])
    }

    fn open_bytes_region(&mut self, plane_index: u32, _x: u32, _y: u32, w: u32, h: u32) -> Result<Vec<u8>> {
        let meta = self.meta.as_ref().ok_or(BioFormatsError::NotInitialized)?;
        if plane_index >= meta.image_count {
            return Err(BioFormatsError::PlaneOutOfRange(plane_index));
        }
        Ok(vec![0u8; w as usize * h as usize * 2])
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

// ---------------------------------------------------------------------------
// 2. Amnis/Luminex IM3 — 64x64 uint16 placeholder
// ---------------------------------------------------------------------------
placeholder_reader_u16_small! {
    /// Amnis/Luminex IM3 format placeholder reader (`.im3`).
    pub struct Im3Reader;
    extensions: ["im3"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 3. 3i SlideBook 7 — 64x64 uint16 placeholder
// ---------------------------------------------------------------------------
placeholder_reader_u16_small! {
    /// 3i SlideBook 7 format placeholder reader (`.sld`).
    pub struct SlideBook7Reader;
    extensions: ["sld"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 4. NDPI Set — 512x512 uint8 placeholder
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// NDPI Set format placeholder reader (`.ndpis`).
    pub struct NdpisReader;
    extensions: ["ndpis"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 5. iVision IPM — 512x512 uint8 placeholder
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// iVision format placeholder reader (`.ipm`).
    pub struct IvisionReader;
    extensions: ["ipm"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 6. Aperio AFI — 512x512 uint8 placeholder
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Aperio AFI fluorescence format placeholder reader (`.afi`).
    pub struct AfiFluorescenceReader;
    extensions: ["afi"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 7. Imaris TIFF — 512x512 uint8 placeholder
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Imaris TIFF format placeholder reader (`.ims`).
    pub struct ImarisTiffReader;
    extensions: ["ims"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 8. Leica XLEF — 512x512 uint8 placeholder
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Leica XLEF format placeholder reader (`.xlef`).
    pub struct XlefReader;
    extensions: ["xlef"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 9. Olympus OIR — 512x512 uint8 placeholder
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Olympus OIR format placeholder reader (`.oir`).
    pub struct OirReader;
    extensions: ["oir"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 10. Olympus cellSens VSI — 512x512 uint8 placeholder
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Olympus cellSens VSI format placeholder reader (`.vsi`).
    pub struct CellSensReader;
    extensions: ["vsi"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 11. Volocity clipping ACFF — 512x512 uint8 placeholder
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Volocity clipping format placeholder reader (`.acff`).
    pub struct VolocityClippingReader;
    extensions: ["acff"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 12. Bruker MicroCT — 512x512 uint8 placeholder
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Bruker MicroCT format placeholder reader (`.ctf`).
    pub struct MicroCtReader;
    extensions: ["ctf"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 13. Bio-Rad SCN confocal — 512x512 uint8 placeholder
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Bio-Rad SCN confocal format placeholder reader (`.scn`).
    pub struct BioRadScnReader;
    extensions: ["scn"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 14. 3i SlideBook TIFF export — 512x512 uint8 placeholder
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// 3i SlideBook TIFF export format placeholder reader (`.tif`).
    pub struct SlidebookTiffReader;
    extensions: ["tif"];
    magic_bytes: false;
}
