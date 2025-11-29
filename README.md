# Camera Record Simulator

Ein Rust-Tool zum Aufnehmen von Kameras und Simulieren virtueller Kameras für Tests - mit grafischer Benutzeroberfläche (GUI) und CLI.

## Features

- **Grafische Benutzeroberfläche (GUI)**: Intuitives GTK4-Interface für alle Funktionen
- **Echte Kamera-Aufnahme**: Nimmt von echten USB-Kameras auf
- **Virtuelle Test-Kameras**: Zwei simulierte Kameras mit Test-Pattern (keine Hardware nötig)
- **Playback-Simulation**: Nutze aufgezeichnete Videos als virtuelle Kameras
  - Wähle einen Ordner mit Aufnahmen
  - Linke Aufnahme → Virtuelle Kamera 0 (links)
  - Rechte Aufnahme → Virtuelle Kamera 1 (rechts)
  - Perfekt für Stereo-Vision Tests ohne echte Hardware!
- **Aufnahme & Wiedergabe**: Speichert Videos und spielt sie ab
- **Metadaten**: Speichert Aufnahme-Informationen als JSON
- **Cross-Distribution**: AppImage und Flatpak für alle Linux-Distros

## Screenshots

GUI mit 3 Tabs:
1. **Aufnahme**: Von echten oder virtuellen Test-Kameras aufnehmen
2. **Simulation**: Ordner mit Aufnahmen als virtuelle Stereo-Kameras laden
3. **Wiedergabe**: Aufnahmen ansehen

## Installation

### Schnellstart (Empfohlen)

**AppImage (funktioniert auf allen Linux-Distros):**
```bash
# Download AppImage (oder baue es selbst, siehe unten)
chmod +x cam_record_sim-x86_64.AppImage
./cam_record_sim-x86_64.AppImage
```

### Selbst bauen

#### Voraussetzungen

**Fedora:**
```bash
sudo dnf install opencv-devel clang gtk4-devel rust cargo
```

**Ubuntu/Debian:**
```bash
sudo apt-get install libopencv-dev clang libclang-dev libgtk-4-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Arch:**
```bash
sudo pacman -S opencv clang gtk4 rust
```

#### Build

```bash
# Alle Packages bauen (AppImage, Flatpak, Native)
chmod +x build-all.sh
./build-all.sh

# Oder nur native Binary
cargo build --release
```

Siehe [BUILD.md](BUILD.md) für detaillierte Build-Anleitung und Distribution-spezifische Packages.

## Verwendung

### GUI-Modus (Standard)

Einfach ohne Parameter starten:
```bash
./cam_record_sim
# oder
./cam_record_sim-x86_64.AppImage
```

#### Tab 1: Aufnahme
- Wähle echte Kamera oder virtuelle Test-Kamera
- Stelle FPS und Dauer ein
- Klicke "Aufnahme starten"

#### Tab 2: Simulation (Hauptfeature!)
1. Nimm zuerst von zwei Kameras auf (oder nutze vorhandene Videos)
2. Gib den Ordner mit den Aufnahmen an (z.B. `recordings/`)
3. Klicke "Laden"
   - Erste Datei wird als **linke virtuelle Kamera** geladen
   - Zweite Datei wird als **rechte virtuelle Kamera** geladen
4. Klicke "Simulation starten"
   - Videos laufen in Endlosschleife
   - Können als echte Kameras verwendet werden
   - Perfekt für Stereo-Vision Tests!

#### Tab 3: Wiedergabe
- Liste alle Aufnahmen auf
- Spiele sie ab

### CLI-Modus

```bash
# Verfügbare Kameras anzeigen
./cam_record_sim list-cameras

# Von echter Kamera aufnehmen
./cam_record_sim record --camera 0 --duration 60

# Von virtueller Test-Kamera aufnehmen (Test-Pattern)
./cam_record_sim sim-record --camera 0 --duration 10

# Beide virtuellen Test-Kameras testen
./cam_record_sim test-virtual --duration 5

# Aufnahmen auflisten
./cam_record_sim list-recordings

# Aufnahme abspielen
./cam_record_sim play camera_0__20240101_120000.avi

# Hilfe
./cam_record_sim --help
```

## Projekt-Struktur

```
src/
├── main.rs              # CLI-Interface und Hauptlogik
├── gui.rs               # GTK4 Grafische Benutzeroberfläche
├── camera.rs            # Echte Kamera-Verwaltung
├── virtual_camera.rs    # Virtuelle Test-Kamera (Pattern-Generator)
├── playback_camera.rs   # Playback als virtuelle Kamera (Simulation!)
├── recorder.rs          # Video-Aufnahme
└── player.rs            # Video-Wiedergabe

packaging/
├── cam_record_sim.desktop           # Desktop Entry
├── cam_record_sim.appdata.xml       # AppStream Metadata
├── com.github.fasttube.CamRecordSim.yml  # Flatpak Manifest
└── icon.png                         # Application Icon

build-all.sh         # Baut alle Packages
build-appimage.sh    # Baut AppImage
build-flatpak.sh     # Baut Flatpak
BUILD.md            # Detaillierte Build-Anleitung
```

## Anwendungsfälle

### 1. Stereo-Vision Entwicklung ohne Hardware
```
1. Nimm einmal von echten Stereo-Kameras auf
2. Entwickle danach mit den Aufnahmen als virtuelle Kameras
3. Keine echte Hardware mehr nötig!
4. Reproduzierbare Tests
```

### 2. Test-Automatisierung
```
- Nutze aufgezeichnete Test-Szenarien
- Wiederhole Tests mit exakt gleichen Inputs
- Continuous Integration möglich
```

### 3. Entwicklung unterwegs
```
- Nimm am Arbeitsplatz (mit Hardware) auf
- Entwickle zu Hause (ohne Hardware) weiter
- Gleiche Code-Basis, simulierte Inputs
```

## Virtuelles Kamera-System

### Test-Pattern Kameras (virtual_camera.rs)
Die virtuellen Test-Kameras generieren bewegende Farbbalken:
- **Kamera 0**: Rote Balken
- **Kamera 1**: Grüne Balken

Gut für: Grundlegende Tests, Pipeline-Validierung

### Playback-Kameras (playback_camera.rs)
Die Playback-Kameras spielen aufgezeichnete Videos ab:
- Nutzt echte aufgenommene Daten
- Endlosschleife (Loop)
- Stereo-System (links + rechts)

Gut für: Realistische Tests, Entwicklung ohne Hardware, Reproduzierbare Szenarien

## Aufnahme-Format

- Video: AVI mit MJPEG Codec
- Metadaten: JSON-Datei mit gleichem Namen
- Auflösung: 640x480 (konfigurierbar)
- Standard FPS: 30 (konfigurierbar)

## Beispiel Workflow: Stereo-Vision Entwicklung

```bash
# 1. Nimm von zwei echten Kameras auf
./cam_record_sim record --camera 0 --duration 30 --output stereo_recordings/
./cam_record_sim record --camera 1 --duration 30 --output stereo_recordings/

# 2. Starte GUI
./cam_record_sim

# 3. Im Simulation-Tab:
#    - Ordner: stereo_recordings/
#    - Klicke "Laden"
#    - Linke Kamera: camera_0__timestamp.avi
#    - Rechte Kamera: camera_1__timestamp.avi
#    - Klicke "Simulation starten"

# 4. Deine Stereo-Vision Software kann jetzt
#    die virtuellen Kameras als Input nutzen!
```

## Distribution

### Für End-User

**Empfohlen: AppImage**
- Funktioniert auf allen Linux-Distros
- Keine Installation nötig
- Einfach herunterladen und ausführen

```bash
chmod +x cam_record_sim-x86_64.AppImage
./cam_record_sim-x86_64.AppImage
```

### Für Entwickler

**Native Build**
```bash
cargo build --release
./target/release/cam_record_sim
```

### Für Sandboxing

**Flatpak**
```bash
flatpak install cam_record_sim.flatpak
flatpak run com.github.fasttube.CamRecordSim
```

Siehe [BUILD.md](BUILD.md) für alle Build-Optionen.

## Troubleshooting

### Kamera wird nicht erkannt
```bash
# Füge User zur video Gruppe hinzu
sudo usermod -a -G video $USER
# Logout/Login erforderlich
```

### GUI startet nicht
```bash
# Prüfe GTK4 Installation
pkg-config --modversion gtk4

# Fedora: sudo dnf install gtk4
# Ubuntu: sudo apt install libgtk-4-1
```

### OpenCV Fehler
```bash
# Prüfe OpenCV
pkg-config --modversion opencv4

# Fedora: sudo dnf install opencv
# Ubuntu: sudo apt install libopencv-core4.5d
```

### Flatpak: Kamera funktioniert nicht
```bash
# Gebe Flatpak Geräte-Zugriff
flatpak override --user --device=all com.github.fasttube.CamRecordSim
```

## Lizenz

MIT License

## Contributing

Pull Requests sind willkommen! Für größere Änderungen bitte zuerst ein Issue öffnen.

## Support

- Issues: https://github.com/fasttube/cam_record_sim/issues
- Dokumentation: [BUILD.md](BUILD.md)
