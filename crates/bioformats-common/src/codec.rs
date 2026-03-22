use crate::error::{BioFormatsError, Result};

/// Decompress LZW-encoded data (TIFF variant — horizontal differencing applied separately).
pub fn decompress_lzw(data: &[u8]) -> Result<Vec<u8>> {
    use weezl::{BitOrder, decode::Decoder};
    let mut decoder = Decoder::with_tiff_size_switch(BitOrder::Msb, 8);
    decoder.decode(data).map_err(|e| BioFormatsError::Codec(e.to_string()))
}

/// Decompress Deflate/Zlib data (TIFF compression 8 = Deflate, 32946 = deflate without header).
pub fn decompress_deflate(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).map_err(BioFormatsError::Io)?;
    Ok(out)
}

/// Decompress raw Deflate (no zlib header).
pub fn decompress_deflate_raw(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::DeflateDecoder;
    use std::io::Read;
    let mut decoder = DeflateDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).map_err(BioFormatsError::Io)?;
    Ok(out)
}

/// Decompress PackBits run-length encoding (TIFF compression 32773).
pub fn decompress_packbits(data: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let header = data[i] as i8;
        i += 1;
        if header >= 0 {
            // Copy (header+1) literal bytes
            let count = (header as usize) + 1;
            if i + count > data.len() {
                return Err(BioFormatsError::InvalidData(
                    "PackBits: literal run overruns input".into(),
                ));
            }
            out.extend_from_slice(&data[i..i + count]);
            i += count;
        } else if header != -128 {
            // Repeat next byte (-header+1) times
            let count = (-header as usize) + 1;
            if i >= data.len() {
                return Err(BioFormatsError::InvalidData(
                    "PackBits: repeat run missing byte".into(),
                ));
            }
            let byte = data[i];
            i += 1;
            for _ in 0..count {
                out.push(byte);
            }
        }
        // header == -128: NOP
    }
    Ok(out)
}

/// Decompress JPEG data.
pub fn decompress_jpeg(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = jpeg_decoder::Decoder::new(data);
    decoder.decode().map_err(|e| BioFormatsError::Codec(e.to_string()))
}

/// Decompress Zstd data.
pub fn decompress_zstd(data: &[u8]) -> Result<Vec<u8>> {
    zstd::decode_all(data).map_err(BioFormatsError::Io)
}

/// Apply TIFF horizontal differencing predictor (predictor = 2).
/// Modifies `data` in-place. `samples_per_pixel` is the number of components.
pub fn undo_horizontal_differencing(data: &mut [u8], samples_per_pixel: usize) {
    if samples_per_pixel == 0 || data.len() < samples_per_pixel * 2 {
        return;
    }
    for i in samples_per_pixel..data.len() {
        data[i] = data[i].wrapping_add(data[i - samples_per_pixel]);
    }
}

/// Apply TIFF horizontal differencing predictor for 16-bit samples.
pub fn undo_horizontal_differencing_u16(data: &mut [u16], samples_per_pixel: usize) {
    if samples_per_pixel == 0 || data.len() < samples_per_pixel * 2 {
        return;
    }
    for i in samples_per_pixel..data.len() {
        data[i] = data[i].wrapping_add(data[i - samples_per_pixel]);
    }
}
