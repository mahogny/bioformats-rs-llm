//! Placeholder readers for miscellaneous / proprietary formats.
//!
//! These readers are extension-only (or magic-byte only for JPEG 2000) and
//! return 512×512 uint8 placeholder metadata with zeroed pixel data.
//! Full decoding is not implemented.

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
// 1. Apple QuickTime
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Apple QuickTime movie placeholder reader (`.mov`, `.qt`).
    pub struct QuickTimeReader;
    extensions: ["mov", "qt"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 2. Multiple-image Network Graphics
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// MNG (Multiple-image Network Graphics) placeholder reader (`.mng`).
    pub struct MngReader;
    extensions: ["mng"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 3. Volocity Library
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Volocity Library placeholder reader (`.acff`).
    pub struct VolocityLibraryReader;
    extensions: ["acff"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 4. 3i SlideBook
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// 3i SlideBook placeholder reader (`.sld`).
    pub struct SlideBookReader;
    extensions: ["sld"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 5. MINC neuroimaging (NetCDF-based)
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// MINC neuroimaging placeholder reader (`.mnc`).
    pub struct MincReader;
    extensions: ["mnc"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 6. PerkinElmer Openlab LIFF
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// PerkinElmer Openlab LIFF placeholder reader (`.liff`).
    pub struct OpenlabLiffReader;
    extensions: ["liff"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 7. JPEG 2000 — magic-byte detection + extension
// ---------------------------------------------------------------------------
/// JPEG 2000 placeholder reader.
///
/// Detects via magic bytes:
/// - `FF 4F FF 51` — JPEG 2000 codestream (J2C)
/// - `00 00 00 0C 6A 50 20 20` — JP2 container
///
/// Returns placeholder metadata; pixel data is zeroed.
pub struct Jpeg2000Reader {
    path: Option<PathBuf>,
    meta: Option<ImageMetadata>,
}

impl Jpeg2000Reader {
    pub fn new() -> Self {
        Jpeg2000Reader { path: None, meta: None }
    }
}

impl Default for Jpeg2000Reader {
    fn default() -> Self { Self::new() }
}

impl FormatReader for Jpeg2000Reader {
    fn is_this_type_by_name(&self, path: &Path) -> bool {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase());
        matches!(ext.as_deref(), Some("jp2") | Some("j2k"))
    }

    fn is_this_type_by_bytes(&self, header: &[u8]) -> bool {
        // J2C codestream: FF 4F FF 51
        if header.len() >= 4 && header[..4] == [0xFF, 0x4F, 0xFF, 0x51] {
            return true;
        }
        // JP2 container: 00 00 00 0C 6A 50 20 20
        if header.len() >= 8 && header[..8] == [0x00, 0x00, 0x00, 0x0C, 0x6A, 0x50, 0x20, 0x20] {
            return true;
        }
        false
    }

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

// ---------------------------------------------------------------------------
// 8. Sedat Lab format
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Sedat Lab format placeholder reader (`.sedat`).
    pub struct SedatReader;
    extensions: ["sedat"];
    magic_bytes: false;
}

// ---------------------------------------------------------------------------
// 9. SM-Camera
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// SM-Camera placeholder reader (`.smc`).
    pub struct SmCameraReader;
    extensions: ["smc"];
    magic_bytes: false;
}
