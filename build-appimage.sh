#!/bin/bash
set -e

echo "Building AppImage for cam_record_sim..."

# Build Release
echo "Step 1/5: Building Release Binary..."
export LIBCLANG_PATH=/lib64  # Für Fedora/RHEL
cargo build --release

# Create AppDir structure
echo "Step 2/5: Creating AppDir structure..."
APPDIR="cam_record_sim.AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPDIR/usr/share/metainfo"

# Copy binary
echo "Step 3/5: Copying binary..."
cp target/release/cam_record_sim "$APPDIR/usr/bin/"

# Copy desktop file
echo "Step 4/5: Copying desktop file and icon..."
cp packaging/cam_record_sim.desktop "$APPDIR/usr/share/applications/"
cp packaging/cam_record_sim.desktop "$APPDIR/"
cp packaging/icon.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/cam_record_sim.png"
cp packaging/icon.png "$APPDIR/cam_record_sim.png"

# Copy AppStream metadata
cp packaging/cam_record_sim.appdata.xml "$APPDIR/usr/share/metainfo/"

# Create AppRun script
cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS}"

exec "${HERE}/usr/bin/cam_record_sim" "$@"
EOF

chmod +x "$APPDIR/AppRun"

# Download appimagetool if not exists
echo "Step 5/5: Building AppImage..."
if [ ! -f "appimagetool-x86_64.AppImage" ]; then
    echo "Downloading appimagetool..."
    wget "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x appimagetool-x86_64.AppImage
fi

# Build AppImage
ARCH=x86_64 ./appimagetool-x86_64.AppImage "$APPDIR" cam_record_sim-x86_64.AppImage

echo "AppImage created: cam_record_sim-x86_64.AppImage"
echo "You can now run: ./cam_record_sim-x86_64.AppImage"
