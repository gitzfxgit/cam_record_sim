#!/bin/bash
set -e

echo "======================================"
echo "Camera Record Sim - Universal Builder"
echo "======================================"
echo ""
echo "This script will build packages for multiple Linux distributions:"
echo "  1. AppImage (universal)"
echo "  2. Flatpak (universal)"
echo "  3. Native binary (for local use)"
echo ""

# Farben für Output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Build native binary
echo -e "${BLUE}[1/3] Building native binary...${NC}"
export LIBCLANG_PATH=/lib64  # Für Fedora/RHEL
cargo build --release
echo -e "${GREEN}✓ Native binary built: target/release/cam_record_sim${NC}"
echo ""

# Build AppImage
echo -e "${BLUE}[2/3] Building AppImage...${NC}"
if [ -f "build-appimage.sh" ]; then
    bash build-appimage.sh
    echo -e "${GREEN}✓ AppImage built: cam_record_sim-x86_64.AppImage${NC}"
else
    echo "Warning: build-appimage.sh not found, skipping AppImage build"
fi
echo ""

# Build Flatpak (optional)
echo -e "${BLUE}[3/3] Building Flatpak...${NC}"
read -p "Build Flatpak? This requires flatpak-builder (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -f "build-flatpak.sh" ]; then
        bash build-flatpak.sh
        echo -e "${GREEN}✓ Flatpak built: cam_record_sim.flatpak${NC}"
    else
        echo "Warning: build-flatpak.sh not found, skipping Flatpak build"
    fi
else
    echo "Skipping Flatpak build"
fi
echo ""

echo "======================================"
echo -e "${GREEN}Build complete!${NC}"
echo "======================================"
echo ""
echo "Available packages:"
echo "  - Native binary: target/release/cam_record_sim"
if [ -f "cam_record_sim-x86_64.AppImage" ]; then
    echo "  - AppImage: cam_record_sim-x86_64.AppImage"
fi
if [ -f "cam_record_sim.flatpak" ]; then
    echo "  - Flatpak: cam_record_sim.flatpak"
fi
echo ""
echo "To run:"
echo "  Native:   ./target/release/cam_record_sim"
echo "  AppImage: ./cam_record_sim-x86_64.AppImage"
echo "  Flatpak:  flatpak install cam_record_sim.flatpak && flatpak run com.github.fasttube.CamRecordSim"
