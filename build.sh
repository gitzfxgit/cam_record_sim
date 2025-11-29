#!/bin/bash

# Build script für cam_record_sim
# Setzt automatisch die nötigen Umgebungsvariablen

echo "Building cam_record_sim..."
echo ""

# Setze LIBCLANG_PATH für Fedora
export LIBCLANG_PATH=/lib64

# Build Release
cargo build --release

echo ""
echo "Build complete!"
echo "Run with: ./target/release/cam_record_sim"
