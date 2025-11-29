# Quick Start Guide

## Schritt 1: Abhängigkeiten installieren

### Fedora (dein System):

```bash
sudo dnf install -y \
    rust \
    cargo \
    gstreamer1-devel \
    gstreamer1-plugins-base-devel \
    gstreamer1-plugins-good \
    gstreamer1-plugins-bad-free \
    gtk4-devel \
    graphene-devel \
    glib2-devel \
    cairo-devel \
    pango-devel \
    gdk-pixbuf2-devel
```

**Oder nutze das Installations-Skript:**
```bash
chmod +x install-dependencies.sh
./install-dependencies.sh
```

## Schritt 2: Projekt bauen

### Einfachste Methode (mit automatischer Umgebungsvariablen-Konfiguration):
```bash
./build.sh
```

### Oder manuell:
```bash
export LIBCLANG_PATH=/lib64
cargo build --release
```

## Schritt 3: Starten

### GUI-Modus (empfohlen):
```bash
./target/release/cam_record_sim
```

### CLI-Modus:
```bash
# Hilfe anzeigen
./target/release/cam_record_sim --help

# Virtuelle Test-Kamera testen
./target/release/cam_record_sim test-virtual --duration 5
```

## Wichtige Pakete erklärt:

- **gstreamer1-devel + plugins**: GStreamer für Video-Aufnahme und -Wiedergabe
- **gtk4-devel + graphene-devel + glib2-devel**: Für GTK4 GUI
- **cairo-devel + pango-devel + gdk-pixbuf2-devel**: GTK4 Abhängigkeiten

## Troubleshooting

### Fehler: "graphene-gobject-1.0 was not found"
```bash
sudo dnf install graphene-devel
```

### Fehler: GStreamer nicht gefunden
```bash
sudo dnf install gstreamer1-devel gstreamer1-plugins-base-devel
```

## Nach erfolgreicher Installation

Das Projekt bietet:

1. **GUI**: Einfach `./target/release/cam_record_sim` starten
   - Tab "Aufnahme": Von Kameras aufnehmen
   - Tab "Simulation": Videos als virtuelle Kameras laden
   - Tab "Wiedergabe": Aufnahmen abspielen

2. **CLI**: Mit `--help` alle Befehle sehen

3. **Build Packages**: Mit `./build-all.sh` AppImage und Flatpak erstellen
