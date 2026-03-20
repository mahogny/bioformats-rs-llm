//! Structured OME metadata — the Rust equivalent of Java Bio-Formats `IMetadata`.
//!
//! Populated by format readers that carry OME-XML or equivalent rich metadata
//! (CZI, OME-TIFF, OME-XML). Readers without this information return `None`
//! from [`crate::reader::FormatReader::ome_metadata`].

// ─── Public types ────────────────────────────────────────────────────────────

/// Top-level metadata store — one [`OmeImage`] per image series.
#[derive(Debug, Clone, Default)]
pub struct OmeMetadata {
    pub images: Vec<OmeImage>,
}

/// Metadata for one image series.
#[derive(Debug, Clone, Default)]
pub struct OmeImage {
    pub name: Option<String>,
    pub description: Option<String>,
    /// Physical pixel size in X (micrometres).
    pub physical_size_x: Option<f64>,
    /// Physical pixel size in Y (micrometres).
    pub physical_size_y: Option<f64>,
    /// Physical pixel size in Z / z-step (micrometres).
    pub physical_size_z: Option<f64>,
    /// Time between frames (seconds).
    pub time_increment: Option<f64>,
    pub channels: Vec<OmeChannel>,
    pub planes: Vec<OmePlane>,
}

/// Per-channel metadata.
#[derive(Debug, Clone, Default)]
pub struct OmeChannel {
    pub name: Option<String>,
    /// Samples (components) per pixel — 1 for greyscale, 3 for RGB.
    pub samples_per_pixel: u32,
    /// Packed RGBA colour as stored in OME-XML (may be negative due to sign).
    pub color: Option<i32>,
    /// Emission wavelength (nm).
    pub emission_wavelength: Option<f64>,
    /// Excitation wavelength (nm).
    pub excitation_wavelength: Option<f64>,
}

/// Per-plane metadata.
#[derive(Debug, Clone, Default)]
pub struct OmePlane {
    pub the_z: u32,
    pub the_c: u32,
    pub the_t: u32,
    /// Time offset from acquisition start (seconds).
    pub delta_t: Option<f64>,
    /// Exposure / integration time (seconds).
    pub exposure_time: Option<f64>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
    pub position_z: Option<f64>,
}

// ─── Parsers ──────────────────────────────────────────────────────────────────

impl OmeMetadata {
    /// Parse OME-XML into structured metadata.
    ///
    /// Handles both standalone `.ome` files and OME-XML embedded in TIFF
    /// `ImageDescription` tags.
    pub fn from_ome_xml(xml: &str) -> Self {
        let mut images = Vec::new();
        let lower_xml = xml.to_ascii_lowercase();

        for img_start in all_tag_positions(xml, "Image") {
            let img_tag = start_tag_at(xml, img_start);
            let name = xml_attr(img_tag, "Name");

            let img_end = lower_xml[img_start..].find("</image>")
                .map(|p| p + img_start + "</image>".len())
                .unwrap_or(xml.len());
            let img_xml = &xml[img_start..img_end];
            let img_lower = img_xml.to_ascii_lowercase();

            let description = xml_inner_text(img_xml, "Description");

            let pixels_pos = all_tag_positions(img_xml, "Pixels").into_iter().next();

            let (physical_size_x, physical_size_y, physical_size_z, time_increment) =
                if let Some(p) = pixels_pos {
                    let pt = start_tag_at(img_xml, p);
                    let psx = xml_attr(pt, "PhysicalSizeX").and_then(|s| s.parse::<f64>().ok());
                    let psy = xml_attr(pt, "PhysicalSizeY").and_then(|s| s.parse::<f64>().ok());
                    let psz = xml_attr(pt, "PhysicalSizeZ").and_then(|s| s.parse::<f64>().ok());
                    let ti  = xml_attr(pt, "TimeIncrement").and_then(|s| s.parse::<f64>().ok());
                    let psx_u = xml_attr(pt, "PhysicalSizeXUnit").unwrap_or_else(|| "µm".into());
                    let psy_u = xml_attr(pt, "PhysicalSizeYUnit").unwrap_or_else(|| "µm".into());
                    let psz_u = xml_attr(pt, "PhysicalSizeZUnit").unwrap_or_else(|| "µm".into());
                    let ti_u  = xml_attr(pt, "TimeIncrementUnit").unwrap_or_else(|| "s".into());
                    (
                        psx.map(|v| to_microns(v, &psx_u)),
                        psy.map(|v| to_microns(v, &psy_u)),
                        psz.map(|v| to_microns(v, &psz_u)),
                        ti.map(|v|  to_seconds(v, &ti_u)),
                    )
                } else {
                    (None, None, None, None)
                };

            // Parse channels and planes from within the <Pixels> block.
            let pixels_end = pixels_pos.and_then(|p| {
                img_lower[p..].find("</pixels>").map(|e| p + e + "</pixels>".len())
            }).unwrap_or(img_xml.len());
            let pixels_xml = pixels_pos.map(|p| &img_xml[p..pixels_end]);

            let channels = pixels_xml.map(parse_channels).unwrap_or_default();
            let planes   = pixels_xml.map(parse_planes).unwrap_or_default();

            images.push(OmeImage {
                name, description,
                physical_size_x, physical_size_y, physical_size_z, time_increment,
                channels, planes,
            });
        }

        OmeMetadata { images }
    }

    /// Parse Zeiss CZI metadata XML into structured metadata.
    pub fn from_czi_xml(xml: &str) -> Self {
        let image = OmeImage {
            physical_size_x: czi_distance(xml, "X"),
            physical_size_y: czi_distance(xml, "Y"),
            physical_size_z: czi_distance(xml, "Z"),
            channels: czi_channels(xml),
            ..Default::default()
        };
        OmeMetadata { images: vec![image] }
    }
}

// ─── OME-XML helpers ─────────────────────────────────────────────────────────

fn parse_channels(pixels_xml: &str) -> Vec<OmeChannel> {
    all_tag_positions(pixels_xml, "Channel").into_iter().map(|pos| {
        let tag = start_tag_at(pixels_xml, pos);
        OmeChannel {
            name: xml_attr(tag, "Name"),
            samples_per_pixel: xml_attr(tag, "SamplesPerPixel")
                .and_then(|s| s.parse().ok()).unwrap_or(1),
            color: xml_attr(tag, "Color").and_then(|s| s.parse::<i32>().ok()),
            emission_wavelength:  xml_attr(tag, "EmissionWavelength").and_then(|s| s.parse().ok()),
            excitation_wavelength: xml_attr(tag, "ExcitationWavelength").and_then(|s| s.parse().ok()),
        }
    }).collect()
}

fn parse_planes(pixels_xml: &str) -> Vec<OmePlane> {
    all_tag_positions(pixels_xml, "Plane").into_iter().map(|pos| {
        let tag = start_tag_at(pixels_xml, pos);
        OmePlane {
            the_z: xml_attr(tag, "TheZ").and_then(|s| s.parse().ok()).unwrap_or(0),
            the_c: xml_attr(tag, "TheC").and_then(|s| s.parse().ok()).unwrap_or(0),
            the_t: xml_attr(tag, "TheT").and_then(|s| s.parse().ok()).unwrap_or(0),
            delta_t:       xml_attr(tag, "DeltaT").and_then(|s| s.parse().ok()),
            exposure_time: xml_attr(tag, "ExposureTime").and_then(|s| s.parse().ok()),
            position_x:   xml_attr(tag, "PositionX").and_then(|s| s.parse().ok()),
            position_y:   xml_attr(tag, "PositionY").and_then(|s| s.parse().ok()),
            position_z:   xml_attr(tag, "PositionZ").and_then(|s| s.parse().ok()),
        }
    }).collect()
}

// ─── CZI XML helpers ──────────────────────────────────────────────────────────

/// Extract a physical size from `<Distance Id="axis"><Value>…</Value></Distance>`.
/// CZI stores values in metres; returns micrometres.
fn czi_distance(xml: &str, axis: &str) -> Option<f64> {
    let lower = xml.to_ascii_lowercase();
    let needle = format!("<distance id=\"{}\">", axis.to_ascii_lowercase());
    let start = lower.find(&needle)?;
    let block_end = lower[start..].find("</distance>")
        .map(|p| p + start).unwrap_or(xml.len());
    let metres: f64 = xml_inner_text(&xml[start..block_end], "Value")?.trim().parse().ok()?;
    Some(metres * 1e6) // m → µm
}

/// Extract channel metadata from CZI `<DisplaySetting><Channels>` block.
fn czi_channels(xml: &str) -> Vec<OmeChannel> {
    let lower = xml.to_ascii_lowercase();
    let open = "<channel ";
    let close = "</channel>";
    let mut channels = Vec::new();
    let mut pos = 0;
    while let Some(rel) = lower[pos..].find(open) {
        let start = pos + rel;
        let end = lower[start..].find(close)
            .map(|e| start + e + close.len())
            .unwrap_or(xml.len());
        let block = &xml[start..end];
        let tag = start_tag_at(block, 0);
        let name = xml_attr(tag, "Name");
        // CZI colours are like "#FFFFA500" (ARGB hex)
        let color = xml_inner_text(block, "Color").and_then(|s| {
            let hex = s.trim().trim_start_matches('#');
            i64::from_str_radix(hex, 16).ok().map(|v| v as i32)
        });
        let emission    = xml_inner_text(block, "EmissionWavelength").and_then(|s| s.trim().parse().ok());
        let excitation  = xml_inner_text(block, "ExcitationWavelength").and_then(|s| s.trim().parse().ok());
        if name.is_some() || color.is_some() {
            channels.push(OmeChannel {
                name, samples_per_pixel: 1, color,
                emission_wavelength: emission,
                excitation_wavelength: excitation,
            });
        }
        pos = end;
    }
    channels
}

// ─── Low-level XML primitives ─────────────────────────────────────────────────

/// Extract the value of `attr` from an XML start-tag string (case-insensitive).
fn xml_attr(tag_text: &str, attr: &str) -> Option<String> {
    let lower = tag_text.to_ascii_lowercase();
    let needle = format!("{}=", attr.to_ascii_lowercase());
    let pos = lower.find(&needle)?;
    let rest = &tag_text[pos + needle.len()..];
    let quote = rest.chars().next()?;
    if quote == '"' || quote == '\'' {
        let inner = &rest[1..];
        let end = inner.find(quote)?;
        Some(inner[..end].to_string())
    } else {
        let end = rest.find(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .unwrap_or(rest.len());
        Some(rest[..end].to_string())
    }
}

/// Return the start-tag string beginning at `pos` (from `<` up to and including `>`).
fn start_tag_at(xml: &str, pos: usize) -> &str {
    let end = xml[pos..].find('>')
        .map(|p| p + pos + 1)
        .unwrap_or(xml.len());
    &xml[pos..end]
}

/// Find the trimmed text content of the first `<tag>…</tag>` (case-insensitive).
fn xml_inner_text(xml: &str, tag: &str) -> Option<String> {
    let lower = xml.to_ascii_lowercase();
    let tag_lc = tag.to_ascii_lowercase();
    let open  = format!("<{}", tag_lc);
    let close = format!("</{}>", tag_lc);
    let tag_start = lower.find(&open)?;
    let content_start = lower[tag_start..].find('>')? + tag_start + 1;
    let content_end   = lower[content_start..].find(&close)? + content_start;
    Some(xml[content_start..content_end].trim().to_string())
}

/// Return byte positions of every `<tag` occurrence (case-insensitive),
/// being careful not to match longer tag names (e.g. `<Channel` vs `<Channels`).
fn all_tag_positions(xml: &str, tag: &str) -> Vec<usize> {
    let lower = xml.to_ascii_lowercase();
    let open  = format!("<{}", tag.to_ascii_lowercase());
    let open_len = open.len();
    let mut positions = Vec::new();
    let mut pos = 0;
    while let Some(rel) = lower[pos..].find(&open) {
        let abs = pos + rel;
        let after = abs + open_len;
        if after < lower.len() {
            let c = lower.as_bytes()[after];
            // Ensure this is not a longer tag name
            if c == b'>' || c == b'/' || c.is_ascii_whitespace() {
                positions.push(abs);
            }
        }
        pos = abs + 1;
    }
    positions
}

// ─── Unit conversions ─────────────────────────────────────────────────────────

fn to_microns(value: f64, unit: &str) -> f64 {
    match unit {
        "m"  => value * 1e6,
        "mm" => value * 1e3,
        "nm" => value * 1e-3,
        "pm" => value * 1e-6,
        _ => value, // assume µm
    }
}

fn to_seconds(value: f64, unit: &str) -> f64 {
    match unit {
        "ms"        => value * 1e-3,
        "µs" | "us" => value * 1e-6,
        "ns"        => value * 1e-9,
        "min"       => value * 60.0,
        "h"         => value * 3600.0,
        _           => value, // assume seconds
    }
}
