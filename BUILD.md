# Build-Anleitung

Dieses Dokument beschreibt, wie du `cam_record_sim` für verschiedene Linux-Distributionen bauen kannst.

## Voraussetzungen

### Alle Distributionen

1. **Rust** (1.70 oder neuer)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **OpenCV** (4.x)

**Fedora:**
```bash
sudo dnf install opencv-devel clang
```

**Ubuntu/Debian:**
```bash
sudo apt-get install libopencv-dev clang libclang-dev
```

**Arch:**
```bash
sudo pacman -S opencv clang
```

3. **GTK4** (für GUI)

**Fedora:**
```bash
sudo dnf install gtk4-devel
```

**Ubuntu/Debian:**
```bash
sudo apt-get install libgtk-4-dev
```

**Arch:**
```bash
sudo pacman -S gtk4
```

## Schnell-Build (Empfohlen)

Der einfachste Weg ist das Universal-Build-Skript:

```bash
chmod +x build-all.sh
./build-all.sh
```

Das erstellt:
- Native Binary
- AppImage (funktioniert auf allen Distros)
- Optional: Flatpak

## Einzelne Build-Methoden

### 1. Native Binary

Für lokale Nutzung oder Distribution-spezifische Packages:

```bash
cargo build --release
```

Binary findet sich in: `target/release/cam_record_sim`

**Starten:**
```bash
# GUI-Modus
./target/release/cam_record_sim

# CLI-Modus
./target/release/cam_record_sim --help
```

### 2. AppImage (Universal Linux Package)

AppImage funktioniert auf **allen** Linux-Distributionen ohne Installation:

```bash
chmod +x build-appimage.sh
./build-appimage.sh
```

**Starten:**
```bash
chmod +x cam_record_sim-x86_64.AppImage
./cam_record_sim-x86_64.AppImage
```

**Vorteile:**
- Keine Installation nötig
- Funktioniert auf allen Distros
- Einfach zu verteilen

### 3. Flatpak (Universal Sandbox)

Flatpak bietet Sandboxing und funktioniert distributionsübergreifend:

```bash
# Installiere flatpak-builder
sudo dnf install flatpak-builder  # Fedora
sudo apt install flatpak-builder   # Ubuntu

# Build
chmod +x build-flatpak.sh
./build-flatpak.sh
```

**Installation:**
```bash
flatpak install cam_record_sim.flatpak
```

**Starten:**
```bash
flatpak run com.github.fasttube.CamRecordSim
```

### 4. Distribution-spezifische Packages

#### Fedora (RPM)

```bash
# Erstelle RPM spec file
rpmdev-setuptree

# Kopiere Quellen
cp -r . ~/rpmbuild/SOURCES/cam_record_sim-0.1.0

# Build RPM
rpmbuild -ba packaging/cam_record_sim.spec

# RPM findet sich in: ~/rpmbuild/RPMS/x86_64/
```

#### Ubuntu/Debian (DEB)

```bash
# Installiere build essentials
sudo apt install build-essential debhelper

# Erstelle Debian Package
dpkg-buildpackage -us -uc

# DEB findet sich im Parent-Directory
```

#### Arch Linux (PKGBUILD)

```bash
# Verwende das PKGBUILD
makepkg -si
```

## Verwendung

### GUI-Modus (Standard)

Einfach ohne Parameter starten:
```bash
./cam_record_sim
```

Das GUI hat 3 Tabs:

1. **Aufnahme**: Von echten oder virtuellen Test-Kameras aufnehmen
2. **Simulation**: Ordner mit Aufnahmen als virtuelle Kameras laden
3. **Wiedergabe**: Aufnahmen abspielen

### CLI-Modus

Mit Subcommands für Automatisierung:

```bash
# Verfügbare Kameras anzeigen
./cam_record_sim list-cameras

# Von echter Kamera aufnehmen
./cam_record_sim record --camera 0 --duration 60

# Von virtueller Test-Kamera aufnehmen
./cam_record_sim sim-record --camera 0 --duration 10

# Test beide virtuellen Kameras
./cam_record_sim test-virtual --duration 5

# Aufnahmen auflisten
./cam_record_sim list-recordings

# Aufnahme abspielen
./cam_record_sim play camera_0__20240101_120000.avi

# Hilfe
./cam_record_sim --help
```

## Simulation-Feature

Die Simulation ermöglicht es, aufgezeichnete Videos als virtuelle Kameras zu verwenden:

1. Nimm von echten Kameras auf (z.B. zwei Kameras für Stereo)
2. Im Simulation-Tab: Wähle den Ordner mit den Aufnahmen
3. Klicke "Laden" - die ersten beiden Videos werden als linke/rechte Kamera geladen
4. Starte Simulation - die Videos laufen in Endlosschleife als virtuelle Kameras

**Anwendungsfälle:**
- Testen von Stereo-Vision Software ohne echte Kameras
- Reproduzierbare Tests mit fixen Inputs
- Entwicklung ohne Hardware-Zugriff

## Troubleshooting

### OpenCV nicht gefunden
```bash
# Prüfe OpenCV Installation
pkg-config --modversion opencv4

# Falls nicht gefunden, installiere opencv-devel/libopencv-dev
```

### GTK4 Fehler
```bash
# Prüfe GTK4
pkg-config --modversion gtk4

# Falls nicht gefunden, installiere gtk4-devel/libgtk-4-dev
```

### Kamera-Zugriff verweigert
```bash
# Füge User zur video Gruppe hinzu
sudo usermod -a -G video $USER

# Logout/Login erforderlich
```

### Flatpak: Kamera funktioniert nicht
```bash
# Gebe Flatpak Geräte-Zugriff
flatpak override --user --device=all com.github.fasttube.CamRecordSim
```

## Cross-Distribution Kompatibilität

| Methode  | Fedora | Ubuntu | Debian | Arch | openSUSE | Alle |
|----------|--------|--------|--------|------|----------|------|
| Native   | ✓      | ✓      | ✓      | ✓    | ✓        | -    |
| AppImage | ✓      | ✓      | ✓      | ✓    | ✓        | ✓    |
| Flatpak  | ✓      | ✓      | ✓      | ✓    | ✓        | ✓    |

**Empfehlung:**
- Für **End-User**: AppImage (einfachste Distribution)
- Für **Entwickler**: Native Build
- Für **Sandboxing**: Flatpak

## Weitere Informationen

- Repository: https://github.com/fasttube/cam_record_sim
- Issues: https://github.com/fasttube/cam_record_sim/issues
- Dokumentation: README.md
