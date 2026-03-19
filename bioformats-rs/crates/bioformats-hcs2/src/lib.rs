//! HCS (High-Content Screening) format readers — group 2.
//!
//! TIFF-based HCS wrappers and extension-only placeholder readers for
//! various plate/HCS acquisition platforms.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bioformats_common::error::{BioFormatsError, Result};
use bioformats_common::metadata::{DimensionOrder, ImageMetadata};
use bioformats_common::pixel_type::PixelType;
use bioformats_common::reader::FormatReader;

// ---------------------------------------------------------------------------
// Shared placeholder metadata (512x512 uint8)
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
// Macro: thin TIFF wrapper (extension-only detection)
// ---------------------------------------------------------------------------
macro_rules! tiff_wrapper {
    (
        $(#[$attr:meta])*
        pub struct $name:ident;
        extensions: [$($ext:literal),+];
    ) => {
        $(#[$attr])*
        pub struct $name {
            inner: bioformats_tiff::TiffReader,
        }

        impl $name {
            pub fn new() -> Self {
                $name { inner: bioformats_tiff::TiffReader::new() }
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
                self.inner.set_id(path)
            }

            fn close(&mut self) -> Result<()> {
                self.inner.close()
            }

            fn series_count(&self) -> usize {
                self.inner.series_count()
            }

            fn set_series(&mut self, s: usize) -> Result<()> {
                self.inner.set_series(s)
            }

            fn series(&self) -> usize {
                self.inner.series()
            }

            fn metadata(&self) -> &ImageMetadata {
                self.inner.metadata()
            }

            fn open_bytes(&mut self, p: u32) -> Result<Vec<u8>> {
                self.inner.open_bytes(p)
            }

            fn open_bytes_region(&mut self, p: u32, x: u32, y: u32, w: u32, h: u32) -> Result<Vec<u8>> {
                self.inner.open_bytes_region(p, x, y, w, h)
            }

            fn open_thumb_bytes(&mut self, p: u32) -> Result<Vec<u8>> {
                self.inner.open_thumb_bytes(p)
            }

            fn resolution_count(&self) -> usize {
                self.inner.resolution_count()
            }

            fn set_resolution(&mut self, level: usize) -> Result<()> {
                self.inner.set_resolution(level)
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Macro: extension-only placeholder reader
// ---------------------------------------------------------------------------
macro_rules! placeholder_reader {
    (
        $(#[$attr:meta])*
        pub struct $name:ident;
        extensions: [$($ext:literal),+];
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

            fn resolution_count(&self) -> usize { 1 }

            fn set_resolution(&mut self, level: usize) -> Result<()> {
                if level != 0 {
                    Err(BioFormatsError::Format(format!("resolution {} out of range", level)))
                } else {
                    Ok(())
                }
            }
        }
    };
}

// ===========================================================================
// TIFF-based HCS wrappers
// ===========================================================================

// ---------------------------------------------------------------------------
// 1. MetaXpress (Molecular Devices) HCS
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// MetaXpress (Molecular Devices) HCS TIFF (`.tif`).
    pub struct MetaxpressTiffReader;
    extensions: ["tif"];
}

// ---------------------------------------------------------------------------
// 2. SimplePCI / HCImage
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// SimplePCI/HCImage TIFF (`.tif`).
    pub struct SimplePciTiffReader;
    extensions: ["tif"];
}

// ---------------------------------------------------------------------------
// 3. Ionpath MIBI-TOF
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// Ionpath MIBI-TOF TIFF (`.tif`).
    pub struct IonpathMibiTiffReader;
    extensions: ["tif"];
}

// ---------------------------------------------------------------------------
// 4. Beckman Coulter MIAS
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// Beckman Coulter MIAS TIFF (`.tif`).
    pub struct MiasTiffReader;
    extensions: ["tif"];
}

// ---------------------------------------------------------------------------
// 5. Trestle whole-slide
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// Trestle whole-slide TIFF (`.tif`).
    pub struct TrestleReader;
    extensions: ["tif"];
}

// ---------------------------------------------------------------------------
// 6. TissueFAXS
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// TissueFAXS TIFF (`.tif`).
    pub struct TissueFaxsReader;
    extensions: ["tif"];
}

// ---------------------------------------------------------------------------
// 7. Mikroscan
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// Mikroscan TIFF (`.tif`).
    pub struct MikroscanTiffReader;
    extensions: ["tif"];
}

// ===========================================================================
// Extension-only placeholder readers
// ===========================================================================

// ---------------------------------------------------------------------------
// 8. BD Biosciences Pathway
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// BD Biosciences Pathway placeholder reader (`.exp`).
    pub struct BdReader;
    extensions: ["exp"];
}

// ---------------------------------------------------------------------------
// 9. PerkinElmer Columbus
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// PerkinElmer Columbus placeholder reader (`.xml`).
    pub struct ColumbusReader;
    extensions: ["xml"];
}

// ---------------------------------------------------------------------------
// 10. PerkinElmer Operetta
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// PerkinElmer Operetta placeholder reader (`.xml`).
    pub struct OperettaReader;
    extensions: ["xml"];
}

// ---------------------------------------------------------------------------
// 11. Olympus ScanR
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Olympus ScanR placeholder reader (`.xml`).
    pub struct ScanrReader;
    extensions: ["xml"];
}

// ---------------------------------------------------------------------------
// 12. Yokogawa CellVoyager
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Yokogawa CellVoyager placeholder reader (`.mes`, `.mlf`).
    pub struct CellVoyagerReader;
    extensions: ["mes", "mlf"];
}

// ---------------------------------------------------------------------------
// 13. Tecan plate reader
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// Tecan plate reader placeholder reader (`.asc`).
    pub struct TecanReader;
    extensions: ["asc"];
}

// ---------------------------------------------------------------------------
// 14. GE InCell 3000
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// GE InCell 3000 placeholder reader (`.xdce`).
    pub struct InCell3000Reader;
    extensions: ["xdce"];
}

// ---------------------------------------------------------------------------
// 15. RCPNL
// ---------------------------------------------------------------------------
placeholder_reader! {
    /// RCPNL placeholder reader (`.rcpnl`).
    pub struct RcpnlReader;
    extensions: ["rcpnl"];
}
