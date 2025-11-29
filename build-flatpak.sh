#!/bin/bash
set -e

echo "Building Flatpak for cam_record_sim..."

# Installiere Flatpak Builder falls nicht vorhanden
if ! command -v flatpak-builder &> /dev/null; then
    echo "flatpak-builder not found. Please install it:"
    echo "  Fedora: sudo dnf install flatpak-builder"
    echo "  Ubuntu: sudo apt install flatpak-builder"
    exit 1
fi

# Generiere cargo-sources.json
echo "Generating cargo sources..."
if ! command -v flatpak-cargo-generator.py &> /dev/null; then
    if [ ! -f "flatpak-cargo-generator.py" ]; then
        echo "Downloading flatpak-cargo-generator..."
        wget https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py
        chmod +x flatpak-cargo-generator.py
    fi
    python3 ./flatpak-cargo-generator.py ./Cargo.lock -o packaging/cargo-sources.json
else
    flatpak-cargo-generator.py ./Cargo.lock -o packaging/cargo-sources.json
fi

# Build Flatpak
echo "Building Flatpak..."
flatpak-builder --force-clean --repo=repo build-dir packaging/com.github.fasttube.CamRecordSim.yml

# Erstelle .flatpak Bundle
echo "Creating Flatpak bundle..."
flatpak build-bundle repo cam_record_sim.flatpak com.github.fasttube.CamRecordSim

echo "Flatpak created: cam_record_sim.flatpak"
echo ""
echo "To install:"
echo "  flatpak install cam_record_sim.flatpak"
echo ""
echo "To run:"
echo "  flatpak run com.github.fasttube.CamRecordSim"
