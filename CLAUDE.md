# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repo contains a pure-Rust reimplementation of the [Bio-Formats](https://www.openmicroscopy.org/bio-formats/) scientific image I/O library. The `bioformats/` directory is the upstream Java reference implementation. The Rust workspace is at the repo root.

## Commands

All commands run from the repo root:

```bash
cargo build                          # Build entire workspace
cargo test                           # Run all tests
cargo test -p bioformats             # Test main facade only
cargo test -p bioformats-tiff        # Test a specific format crate
cargo test -p bioformats -- format_tests  # Run a specific test module
```

The Java Bio-Formats source in `bioformats/` is read-only reference — do not modify it.

## Architecture

### Crate Structure

- **`bioformats-common`** — Shared traits and types. All format crates depend only on this. Key modules:
  - `reader.rs` / `writer.rs` — `FormatReader` / `FormatWriter` traits to implement for new formats
  - `metadata.rs` — `ImageMetadata`, `PixelType`, `LookupTable`, `MetadataValue`
  - `codec.rs` — Compression/decompression (LZW, Deflate, PackBits, JPEG, Zstd)
  - `endian.rs` — Byte-order utilities

- **`bioformats`** — Public facade crate. Exposes `ImageReader` and `ImageWriter` (auto-detecting), re-exports common types. Format detection logic lives in `registry.rs` (magic bytes first, then extension) and `writer_registry.rs`.

- **`bioformats-tiff`** — TIFF/BigTIFF implementation. Exposes its `ifd` and `parser` modules publicly so TIFF-based formats (LSM, MetaMorph, SVS, Flex, etc.) can reuse IFD parsing without duplication.

- **72 format crates** — Each implements `FormatReader` and/or `FormatWriter` from `bioformats-common`. Organized by development phase; later phases contain read-only stubs for less common formats.

### Adding a Format

1. Create a new crate in `crates/bioformats-<name>/`
2. Implement `FormatReader` and/or `FormatWriter` from `bioformats-common`
3. Register in `bioformats/src/registry.rs` (magic bytes + extension) and/or `writer_registry.rs`
4. Add as a workspace member in `Cargo.toml` and as a dependency of the `bioformats` crate

### Key Design Decisions

- **No JVM, no native deps** — pure Rust only (some optional workspace deps: hdf5, zstd, etc.)
- **Metadata is strongly typed** — `ImageMetadata` structs, not OME-XML strings
- **Pixel data is raw `Vec<u8>`** — callers interpret bytes according to `PixelType`
- **Multi-series support** — `set_series()` switches context for formats like LIF and ND2
- **TIFF is central** — many microscopy formats are TIFF variants; `bioformats-tiff` is designed for reuse

### Tests

Integration tests and round-trip tests are in `crates/bioformats/tests/`. Fixture files (small test images) are in `tests/fixtures/`. Round-trip tests write a file, read it back, and verify data integrity and metadata fields.
