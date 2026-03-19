//! `bioformats` — pure-Rust reader for scientific image formats.
//!
//! # Quick start
//!
//! ```no_run
//! use bioformats::ImageReader;
//! use std::path::Path;
//!
//! let mut reader = ImageReader::open(Path::new("image.tif")).unwrap();
//! let meta = reader.metadata();
//! println!("{}x{}", meta.size_x, meta.size_y);
//! let plane0 = reader.open_bytes(0).unwrap();
//! ```

pub mod error;
pub mod metadata;
pub mod pixel;
pub mod reader;
pub mod registry;

pub use error::{BioFormatsError, Result};
pub use metadata::{DimensionOrder, ImageMetadata, LookupTable, MetadataValue};
pub use pixel::PixelType;
pub use reader::FormatReader;
pub use registry::ImageReader;
