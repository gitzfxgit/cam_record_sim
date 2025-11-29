================================================================================
   CAMERA RECORD SIMULATOR - Quick Start (AKTUALISIERT!)
================================================================================

WICHTIG: Wir nutzen jetzt GStreamer statt OpenCV - viel einfacher!

SCHRITT 1 - Dependencies installieren:
---------------------------------------
sudo dnf install -y \
    rust cargo \
    gstreamer1-devel \
    gstreamer1-plugins-base-devel \
    gstreamer1-plugins-good \
    gstreamer1-plugins-bad-free \
    gtk4-devel graphene-devel \
    glib2-devel cairo-devel pango-devel gdk-pixbuf2-devel

ODER nutze das Skript:
./install-dependencies.sh

SCHRITT 2 - Build:
------------------
./build.sh              # Build
# ODER
cargo build --release   # Kein LIBCLANG_PATH mehr nötig!

SCHRITT 3 - Start:
------------------
./target/release/cam_record_sim  # GUI starten

================================================================================

WARUM DIE ÄNDERUNG?
- OpenCV brauchte komplizierte libclang Setup
- GStreamer ist einfacher und besser für Video
- Keine Umgebungsvariablen mehr nötig!

WAS WIR NUTZEN:
- GStreamer: Video-Aufnahme und -Wiedergabe
- GTK4: Grafisches Interface
- Nokhwa: Kamera-Zugriff

DOKUMENTATION:
- QUICK_START.md  - Schnellstart-Anleitung
- README.md       - Vollständige Dokumentation
- BUILD.md        - Build für alle Distributionen

================================================================================
