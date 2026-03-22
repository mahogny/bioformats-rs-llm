//! Thin TIFF-wrapper readers for formats that are TIFF-based but identified
//! only by file extension (no distinct magic bytes beyond TIFF itself).
//!
//! All readers delegate all pixel / metadata work to `crate::tiff::TiffReader`.

use std::path::Path;

use crate::common::error::Result;
use crate::common::metadata::ImageMetadata;
use crate::common::reader::FormatReader;

// ---------------------------------------------------------------------------
// Macro to generate a thin TIFF-wrapper reader.
// ---------------------------------------------------------------------------
macro_rules! tiff_wrapper {
    (
        $(#[$attr:meta])*
        pub struct $name:ident;
        extensions: [$($ext:literal),+];
    ) => {
        $(#[$attr])*
        pub struct $name {
            inner: crate::tiff::TiffReader,
        }

        impl $name {
            pub fn new() -> Self {
                $name { inner: crate::tiff::TiffReader::new() }
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
// 1. Hamamatsu NDPI whole-slide
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// Hamamatsu NDPI whole-slide image (TIFF-based, `.ndpi`).
    pub struct NdpiReader;
    extensions: ["ndpi"];
}

// ---------------------------------------------------------------------------
// 2. Leica SCN whole-slide
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// Leica SCN whole-slide image (TIFF-based, `.scn`).
    pub struct LeicaScnReader;
    extensions: ["scn"];
}

// ---------------------------------------------------------------------------
// 3. Ventana/Roche BIF whole-slide
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// Ventana/Roche BIF whole-slide image (TIFF-based, `.bif`).
    pub struct VentanaReader;
    extensions: ["bif"];
}

// ---------------------------------------------------------------------------
// 4. Nikon NIS-Elements TIFF (metadata embedded in TIFF description)
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// Nikon NIS-Elements annotated TIFF (`.tiff`).
    pub struct NikonElementsTiffReader;
    extensions: ["tiff"];
}

// ---------------------------------------------------------------------------
// 5. FEI-annotated TIFF (extension-only fallback for `.tiff`)
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// FEI-annotated TIFF (extension-only fallback, `.tiff`).
    pub struct FeiTiffReader;
    extensions: ["tiff"];
}

// ---------------------------------------------------------------------------
// 6. Olympus SIS TIFF metadata (`.tif`)
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// Olympus SIS TIFF (`.tif`).
    pub struct OlympusSisTiffReader;
    extensions: ["tif"];
}

// ---------------------------------------------------------------------------
// 7. Improvision/Volocity annotated TIFF (`.tif`)
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// Improvision/Volocity annotated TIFF (`.tif`).
    pub struct ImprovisionTiffReader;
    extensions: ["tif"];
}

// ---------------------------------------------------------------------------
// 8. Zeiss ApoTome TIFF (`.tif`)
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// Zeiss ApoTome TIFF (`.tif`).
    pub struct ZeissApotomeTiffReader;
    extensions: ["tif"];
}

// ---------------------------------------------------------------------------
// 9. Olympus Fluoview FV300 (`.tif`)
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// Olympus Fluoview FV300 TIFF (`.tif`).
    pub struct FluoviewTiffReader;
    extensions: ["tif"];
}

// ---------------------------------------------------------------------------
// 10. Molecular Devices plate TIFF (`.tif`)
// ---------------------------------------------------------------------------
tiff_wrapper! {
    /// Molecular Devices plate TIFF (`.tif`).
    pub struct MolecularDevicesTiffReader;
    extensions: ["tif"];
}
