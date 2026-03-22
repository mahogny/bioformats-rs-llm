use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

use crate::common::error::{BioFormatsError, Result};
use crate::common::endian::*;

use super::ifd::{Ifd, IfdValue};

/// Whether the file is standard (32-bit offsets) or BigTIFF (64-bit offsets).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TiffVariant {
    Classic,
    Big,
}

/// Parsed state of the TIFF stream header.
pub struct TiffParser<R: Read + Seek> {
    pub reader: R,
    pub little_endian: bool,
    pub variant: TiffVariant,
    /// Offset of the first IFD.
    pub first_ifd_offset: u64,
}

impl<R: Read + Seek> TiffParser<R> {
    /// Parse the TIFF/BigTIFF file header.
    pub fn new(mut reader: R) -> Result<Self> {
        reader.seek(SeekFrom::Start(0))?;
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        let little_endian = match &magic[0..2] {
            b"II" => true,
            b"MM" => false,
            _ => {
                return Err(BioFormatsError::Format(
                    "Not a TIFF file: bad byte-order mark".into(),
                ))
            }
        };

        let bigtiff_magic: u16 = if little_endian {
            u16::from_le_bytes([magic[2], magic[3]])
        } else {
            u16::from_be_bytes([magic[2], magic[3]])
        };

        let (variant, first_ifd_offset) = match bigtiff_magic {
            42 => {
                // Classic TIFF
                let mut off_bytes = [0u8; 4];
                reader.read_exact(&mut off_bytes)?;
                let off = if little_endian {
                    u32::from_le_bytes(off_bytes)
                } else {
                    u32::from_be_bytes(off_bytes)
                };
                (TiffVariant::Classic, off as u64)
            }
            43 => {
                // BigTIFF — 2 extra header fields before IFD offset
                let _bytesize = read_u16(&mut reader, little_endian)?; // always 8
                let _always_zero = read_u16(&mut reader, little_endian)?; // always 0
                let off = read_u64(&mut reader, little_endian)?;
                (TiffVariant::Big, off)
            }
            other => {
                return Err(BioFormatsError::Format(format!(
                    "Not a TIFF file: unknown magic {:#06x}",
                    other
                )))
            }
        };

        Ok(TiffParser {
            reader,
            little_endian,
            variant,
            first_ifd_offset,
        })
    }

    /// Read all IFDs in the main IFD chain.
    pub fn read_ifds(&mut self) -> Result<Vec<Ifd>> {
        let mut ifds = Vec::new();
        let mut offset = self.first_ifd_offset;
        while offset != 0 {
            let (ifd, next) = self.read_ifd(offset)?;
            ifds.push(ifd);
            offset = next;
        }
        Ok(ifds)
    }

    /// Read one IFD at `offset`; return the IFD and the offset of the next IFD.
    pub fn read_ifd(&mut self, offset: u64) -> Result<(Ifd, u64)> {
        self.reader.seek(SeekFrom::Start(offset))?;

        let entry_count = if self.variant == TiffVariant::Big {
            read_u64(&mut self.reader, self.little_endian)? as usize
        } else {
            read_u16(&mut self.reader, self.little_endian)? as usize
        };

        let mut entries = HashMap::new();

        for _ in 0..entry_count {
            let tag = read_u16(&mut self.reader, self.little_endian)?;
            let type_code = read_u16(&mut self.reader, self.little_endian)?;
            let (count, value_or_offset) = if self.variant == TiffVariant::Big {
                let c = read_u64(&mut self.reader, self.little_endian)?;
                let v = read_u64(&mut self.reader, self.little_endian)?;
                (c, v)
            } else {
                let c = read_u32(&mut self.reader, self.little_endian)? as u64;
                let v = read_u32(&mut self.reader, self.little_endian)? as u64;
                (c, v)
            };

            if let Ok(value) = self.read_ifd_value(type_code, count, value_or_offset) {
                entries.insert(tag, value);
            }
        }

        // Read next-IFD offset
        let next_ifd = if self.variant == TiffVariant::Big {
            read_u64(&mut self.reader, self.little_endian)?
        } else {
            read_u32(&mut self.reader, self.little_endian)? as u64
        };

        Ok((Ifd { entries }, next_ifd))
    }

    fn read_ifd_value(
        &mut self,
        type_code: u16,
        count: u64,
        value_or_offset: u64,
    ) -> Result<IfdValue> {
        let type_size: u64 = match type_code {
            1 | 2 | 6 | 7 => 1, // BYTE, ASCII, SBYTE, UNDEFINED
            3 | 8 => 2,          // SHORT, SSHORT
            4 | 9 | 13 => 4,     // LONG, SLONG, IFD
            5 | 10 => 8,         // RATIONAL, SRATIONAL
            11 => 4,             // FLOAT
            12 => 8,             // DOUBLE
            16 | 18 => 8,        // LONG8, IFD8 (BigTIFF)
            17 => 8,             // SLONG8 (BigTIFF)
            _ => return Err(BioFormatsError::Format(format!("Unknown IFD type {}", type_code))),
        };

        let total_bytes = count * type_size;

        // Determine if value fits inline or must be read from an offset.
        let inline_limit: u64 = if self.variant == TiffVariant::Big { 8 } else { 4 };

        let data = if total_bytes <= inline_limit {
            // Value is stored inline in the value_or_offset field (little- or big-endian bytes).
            let raw = value_or_offset.to_le_bytes(); // stored LE regardless of file endian
            // Re-interpret inline bytes in file endian order
            let bytes: Vec<u8> = if self.little_endian {
                raw[..total_bytes as usize].to_vec()
            } else {
                // For big-endian files the bytes are left-justified in the field
                raw[..total_bytes as usize].to_vec()
            };
            bytes
        } else {
            let pos_after_entry = self.reader.stream_position()?;
            self.reader.seek(SeekFrom::Start(value_or_offset))?;
            let mut buf = vec![0u8; total_bytes as usize];
            self.reader.read_exact(&mut buf)?;
            self.reader.seek(SeekFrom::Start(pos_after_entry))?;
            buf
        };

        self.decode_ifd_value(type_code, count as usize, &data)
    }

    fn decode_ifd_value(&self, type_code: u16, count: usize, data: &[u8]) -> Result<IfdValue> {
        let le = self.little_endian;
        Ok(match type_code {
            1 => IfdValue::Byte(data.to_vec()),
            2 => {
                // ASCII: null-separated strings; take first
                let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
                IfdValue::Ascii(String::from_utf8_lossy(&data[..end]).into_owned())
            }
            3 => IfdValue::Short(
                data.chunks_exact(2)
                    .map(|c| if le { u16::from_le_bytes([c[0], c[1]]) } else { u16::from_be_bytes([c[0], c[1]]) })
                    .collect(),
            ),
            4 | 13 => IfdValue::Long(
                data.chunks_exact(4)
                    .map(|c| if le { u32::from_le_bytes([c[0], c[1], c[2], c[3]]) } else { u32::from_be_bytes([c[0], c[1], c[2], c[3]]) })
                    .collect(),
            ),
            5 => IfdValue::Rational(
                data.chunks_exact(8)
                    .map(|c| {
                        let n = if le { u32::from_le_bytes([c[0], c[1], c[2], c[3]]) } else { u32::from_be_bytes([c[0], c[1], c[2], c[3]]) };
                        let d = if le { u32::from_le_bytes([c[4], c[5], c[6], c[7]]) } else { u32::from_be_bytes([c[4], c[5], c[6], c[7]]) };
                        (n, d)
                    })
                    .collect(),
            ),
            6 => IfdValue::SByte(data.iter().map(|&b| b as i8).collect()),
            7 => IfdValue::Undefined(data.to_vec()),
            8 => IfdValue::SShort(
                data.chunks_exact(2)
                    .map(|c| if le { i16::from_le_bytes([c[0], c[1]]) } else { i16::from_be_bytes([c[0], c[1]]) })
                    .collect(),
            ),
            9 => IfdValue::SLong(
                data.chunks_exact(4)
                    .map(|c| if le { i32::from_le_bytes([c[0], c[1], c[2], c[3]]) } else { i32::from_be_bytes([c[0], c[1], c[2], c[3]]) })
                    .collect(),
            ),
            10 => IfdValue::SRational(
                data.chunks_exact(8)
                    .map(|c| {
                        let n = if le { i32::from_le_bytes([c[0], c[1], c[2], c[3]]) } else { i32::from_be_bytes([c[0], c[1], c[2], c[3]]) };
                        let d = if le { i32::from_le_bytes([c[4], c[5], c[6], c[7]]) } else { i32::from_be_bytes([c[4], c[5], c[6], c[7]]) };
                        (n, d)
                    })
                    .collect(),
            ),
            11 => IfdValue::Float(
                data.chunks_exact(4)
                    .map(|c| f32::from_bits(if le { u32::from_le_bytes([c[0], c[1], c[2], c[3]]) } else { u32::from_be_bytes([c[0], c[1], c[2], c[3]]) }))
                    .collect(),
            ),
            12 => IfdValue::Double(
                data.chunks_exact(8)
                    .map(|c| f64::from_bits(if le { u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]) } else { u64::from_be_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]) }))
                    .collect(),
            ),
            16 | 18 => IfdValue::Long8(
                data.chunks_exact(8)
                    .map(|c| if le { u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]) } else { u64::from_be_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]) })
                    .collect(),
            ),
            _ => {
                let _ = count;
                IfdValue::Undefined(data.to_vec())
            }
        })
    }
}
