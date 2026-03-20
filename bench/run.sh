#!/usr/bin/env bash
# bench/run.sh — compare Bio-Formats Java vs Rust for reading an OME-TIFF
#
# Usage: ./bench/run.sh [path/to/file.ome.tif]
# Default file: test/tubhiswt_C0.ome.tif

set -euo pipefail
cd "$(dirname "$0")/.."          # repo root

FILE="${1:-test/tubhiswt_C0.ome.tif}"
JAR="bioformats_package.jar"
WARMUP=3
MEASURE=10

if [[ ! -f "$FILE" ]]; then
  echo "ERROR: file not found: $FILE" >&2; exit 1
fi
if [[ ! -f "$JAR" ]]; then
  echo "ERROR: $JAR not found in repo root" >&2; exit 1
fi

# ── file info ──────────────────────────────────────────────────────────────────
FILE_BYTES=$(wc -c < "$FILE" | tr -d ' ')
FILE_MIB=$(echo "scale=1; $FILE_BYTES/1048576" | bc)
echo "File : $FILE  (${FILE_MIB} MiB)"
echo "Runs : ${WARMUP} warmup + ${MEASURE} measured"
echo

# ── build Rust binary ──────────────────────────────────────────────────────────
echo "Building Rust binary (release)..."
cargo build --release --manifest-path bench/Cargo.toml -q
RUST_BIN=bench/target/release/bench_rust
echo

# ── build Java class ───────────────────────────────────────────────────────────
CLASS_DIR=bench/target/java
mkdir -p "$CLASS_DIR"
if [[ ! -f "$CLASS_DIR/BfBench.class" ]]; then
  echo "Compiling BfBench.java..."
  javac -cp "$JAR" bench/BfBench.java -d "$CLASS_DIR"
fi
echo

# ── run ────────────────────────────────────────────────────────────────────────
echo "Running Java Bio-Formats..."
JAVA_NS=$(java -cp "$JAR:$CLASS_DIR" BfBench "$FILE" "$WARMUP" "$MEASURE" 2>/dev/null)

echo "Running Rust bioformats..."
RUST_NS=$("$RUST_BIN" "$FILE" "$WARMUP" "$MEASURE")

# ── report ─────────────────────────────────────────────────────────────────────
to_ms()  { echo "scale=2; $1/1000000"   | bc; }
to_mbs() {
  local ns=$1 bytes=$2
  echo "scale=1; ($bytes * 1000.0) / ($ns / 1000000.0) / 1048576.0" | bc
}

JAVA_MS=$(to_ms  "$JAVA_NS")
RUST_MS=$(to_ms  "$RUST_NS")
JAVA_MBS=$(to_mbs "$JAVA_NS" "$FILE_BYTES")
RUST_MBS=$(to_mbs "$RUST_NS" "$FILE_BYTES")

# Speedup (lower time = faster)
if (( JAVA_NS > RUST_NS )); then
  SPEEDUP=$(echo "scale=2; $JAVA_NS / $RUST_NS" | bc)
  WINNER="Rust is ${SPEEDUP}x faster"
else
  SPEEDUP=$(echo "scale=2; $RUST_NS / $JAVA_NS" | bc)
  WINNER="Java is ${SPEEDUP}x faster"
fi

echo
echo "══════════════════════════════════════════════"
printf "  %-8s  %8s ms   %6s MiB/s\n" "Java"  "$JAVA_MS" "$JAVA_MBS"
printf "  %-8s  %8s ms   %6s MiB/s\n" "Rust"  "$RUST_MS" "$RUST_MBS"
echo "──────────────────────────────────────────────"
echo "  $WINNER"
echo "══════════════════════════════════════════════"
