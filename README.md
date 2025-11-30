# Camera Record Simulator

Dual-Kamera Aufnahme- und Simulationssystem mit GTK4 GUI. Nimmt von echten Kameras auf und spielt die Aufnahmen in einer Endlosschleife als virtuelle Kameras ab - perfekt für Stereo-Vision Entwicklung und Tests ohne Hardware.

## Überblick

Dieses Programm bietet zwei Hauptfunktionen:

1. **Aufnahme-Tab**: Zeichnet von 1-2 echten Kameras auf und speichert Videos in einem konfigurierbaren Ordner
2. **Simulations-Tab**: Lädt aufgenommene Videos und spielt sie in einer Schleife ab, als wären 2 echte Kameras angeschlossen

## Features

- **Dual-Kamera Aufnahme**: Simultane Aufnahme von zwei Kameras mit Live-Preview
- **Video-Simulation**: Gespeicherte Videos als virtuelle Kameras abspielen (Endlosschleife)
- **Live-Vorschau**: Echtzeit-Anzeige beider Kamera-Feeds während Aufnahme und Simulation
- **Flexible Konfiguration**: FPS, Auflösung, Dauer und Ausgabeordner anpassbar
- **Metadaten-Export**: Automatische JSON-Metadaten für jede Aufnahme
- **GTK4 GUI**: Moderne, intuitive Benutzeroberfläche
- **CLI-Unterstützung**: Vollständige Kommandozeilen-Schnittstelle

## Systemanforderungen

### Abhängigkeiten

**GStreamer** (für Video-Encoding/Decoding):
- gstreamer
- gstreamer-plugins-base
- gstreamer-plugins-good
- gstreamer-plugins-bad (enthält openh264enc für H.264 Encoding)

**GTK4** (für GUI):
- gtk4
- glib

**Rust** (zum Kompilieren):
- Rust 2024 Edition
- cargo

### Installation der Abhängigkeiten

**Fedora:**
```bash
./install-dependencies.sh
```

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install \
    gstreamer1.0-tools \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    libgtk-4-dev \
    libclang-dev \
    pkg-config
```

**Arch Linux:**
```bash
sudo pacman -S gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gtk4 clang
```

## Kompilieren

```bash
cargo build --release
./target/release/cam_record_sim
```

Oder direkt ausführen:
```bash
cargo run --release
```

## Verwendung

### GUI-Modus (Standard)

Starte das Programm ohne Parameter:
```bash
./cam_record_sim
```

#### Aufnahme-Tab

1. **Kamera-Auswahl**:
   - Wähle "1 Kamera" oder "2 Kameras"
   - Gib die Kamera-IDs an (normalerweise 0 und 1)

2. **Einstellungen**:
   - FPS: Frames pro Sekunde (Standard: 30)
   - Dauer: Aufnahmedauer in Sekunden (Standard: 10)
   - Ausgabe: Zielordner für Videos (Standard: recordings)

3. **Aufnahme**:
   - Klicke "Aufnahme starten"
   - Live-Vorschau zeigt beide Kamera-Feeds
   - Videos werden mit Zeitstempel gespeichert: `camera_0__YYYYMMDD_HHMMSS.mp4`

#### Simulations-Tab

1. **Videos laden**:
   - Gib den Ordner mit Aufnahmen ein (z.B. "recordings")
   - Klicke "Laden"
   - Die ersten beiden MP4-Dateien werden geladen:
     - Erste Datei → Virtuelle Kamera 0 (Links)
     - Zweite Datei → Virtuelle Kamera 1 (Rechts)

2. **Simulation starten**:
   - Klicke "Simulation starten"
   - Beide Videos werden in einer Endlosschleife abgespielt
   - Live-Vorschau zeigt beide Video-Feeds
   - Die Videos können nun von anderen Anwendungen als Kamera-Input verwendet werden

3. **Simulation stoppen**:
   - Klicke "Simulation stoppen"

### CLI-Modus

Alle Funktionen sind auch über die Kommandozeile verfügbar:

#### Kameras auflisten
```bash
./cam_record_sim list-cameras
```

#### Von echter Kamera aufnehmen
```bash
./cam_record_sim record \
    --camera 0 \
    --output recordings \
    --fps 30.0 \
    --duration 60
```

#### Von virtueller Test-Kamera aufnehmen
```bash
./cam_record_sim sim-record \
    --camera 0 \
    --output recordings \
    --fps 30.0 \
    --duration 10
```

#### Virtuelle Kameras testen
```bash
./cam_record_sim test-virtual --duration 5
```

#### Aufnahmen auflisten
```bash
./cam_record_sim list-recordings --dir recordings
```

#### Aufnahme abspielen
```bash
./cam_record_sim play \
    --file camera_0__20241130_120000.mp4 \
    --dir recordings
```

## Architektur

### Module

#### `main.rs`
- CLI-Argument-Parsing mit clap
- Haupt-Entry-Point
- Routing zwischen GUI und CLI-Modi

#### `gui.rs`
- GTK4 Benutzeroberfläche
- Drei Tabs: Aufnahme, Simulation, Wiedergabe
- Event-Handling und UI-Updates
- Live-Preview-Rendering

#### `camera.rs`
- Verwaltung echter Kameras
- Basiert auf nokhwa-Bibliothek
- Frame-Capturing in RGB-Format

#### `virtual_camera.rs`
- Generiert Test-Pattern (bewegende Farbbalken)
- Nützlich für Tests ohne echte Hardware
- Verschiedene Farben je Kamera-ID

#### `playback_camera.rs`
- Spielt Videos als virtuelle Kameras ab
- GStreamer-basierte Video-Dekodierung
- Unterstützt Loop-Modus für Endlosschleife
- `StereoPlaybackSystem`: Verwaltet linke und rechte Kamera

#### `recorder.rs`
- Video-Aufnahme mit GStreamer
- H.264-Encoding (openh264enc)
- MP4-Container
- Frame-by-Frame-Schreiben
- Metadaten-Export

#### `dual_recorder.rs`
- Koordiniert Aufnahme von mehreren Kameras
- Unterstützt:
  - `CameraSource::Single`: Eine Kamera
  - `CameraSource::Dual`: Zwei echte Kameras
  - `CameraSource::Virtual`: Zwei Test-Kameras
- Thread-basierte asynchrone Aufnahme

#### `player.rs`
- Video-Wiedergabe
- GStreamer-basiert
- Unterstützt MP4, AVI, MKV

### GStreamer Pipelines

#### Aufnahme-Pipeline
```
appsrc → videoconvert → video/x-raw,format=I420 → openh264enc → h264parse → mp4mux → filesink
```

**Parameter**:
- Input: RGB Frames von Kamera
- `videoconvert`: Konvertiert RGB zu I420 (YUV)
- `video/x-raw,format=I420`: Explizite I420-Caps (erforderlich für openh264enc)
- `openh264enc bitrate=2000000`: 2 Mbps Bitrate für gute Qualität
- `h264parse`: Konvertiert byte-stream zu avc format für mp4mux
- `video/x-h264,stream-format=avc`: AVC Format für MP4-Container
- Output: H.264 in MP4-Container

#### Playback-Pipeline
```
filesrc → decodebin → videoconvert → appsink
```

**Parameter**:
- `decodebin`: Automatische Format-Erkennung
- `video/x-raw,format=RGB`: RGB-Output für Frame-Extraction

#### Wiedergabe-Pipeline
```
filesrc → decodebin → videoconvert → autovideosink
```

## Technische Details

### Video-Format

**Encoding**:
- Codec: H.264 (openh264enc)
- Container: MP4
- Pixel-Format: RGB (Input), I420 (Encoding)
- Auflösung: 640x480 (Standard)
- FPS: 30 (konfigurierbar)
- Bitrate: 2 Mbps (Standard)

**Metadaten** (JSON):
```json
{
  "camera_id": 0,
  "timestamp": "2024-11-30T12:00:00+01:00",
  "duration_secs": 10.5,
  "fps": 30.0,
  "width": 640,
  "height": 480,
  "filename": "camera_0__20241130_120000.mp4"
}
```

### Threading-Modell

- **Haupt-Thread**: GTK Event Loop
- **Aufnahme-Thread**: Asynchrone Kamera-Aufnahme und Video-Encoding
- **UI-Update**: glib::timeout_add_local für Live-Preview (30 FPS)

### Frame-Flow

#### Aufnahme
```
Kamera → get_frame() → VideoRecorder → x264enc → MP4-Datei
         ↓
    Live-Preview (GUI)
```

#### Simulation
```
MP4-Datei → PlaybackCamera → get_frame() → Live-Preview (GUI)
                                           ↓
                                    Andere Anwendung
```

## Anwendungsfälle

### 1. Stereo-Vision Entwicklung

**Problem**: Hardware-Setup (zwei Kameras) ist unpraktisch während Entwicklung

**Lösung**:
1. Einmalige Aufnahme von beiden Kameras im Aufnahme-Tab
2. Simulation der Kameras im Simulations-Tab
3. Entwicklung mit reproduzierbaren Video-Inputs
4. Keine Hardware mehr nötig

### 2. Continuous Integration

**Problem**: CI-Server haben keine Kamera-Hardware

**Lösung**:
1. Aufnahmen im Repository speichern
2. Tests verwenden Playback-Kameras statt echter Hardware
3. Reproduzierbare, deterministische Tests

### 3. Offline-Entwicklung

**Problem**: Kamera-Hardware nicht immer verfügbar

**Lösung**:
1. Aufnahmen einmal mit Hardware erstellen
2. Später ohne Hardware weiterentwickeln
3. Gleiche Code-Basis, simulierte Inputs

## Problemlösungen

### GStreamer Pipeline-Fehler

**Fehler 1**: `no element "x264enc"`
- **Ursache**: x264-Plugin nicht installiert (erfordert gst-plugins-ugly)
- **Lösung**: openh264enc verwenden (in gst-plugins-bad enthalten)

**Fehler 2**: `could not link openh264enc0 to mp4mux0`
- **Ursache**: openh264enc gibt byte-stream aus, mp4mux braucht avc format
- **Lösung**: h264parse zwischen openh264enc und mp4mux einfügen

**Fehler 3**: `[OpenH264] Error:CWelsH264SVCEncoder::EncodeFrame(), cmInitParaError`
- **Ursache**: openh264enc akzeptiert nur I420 (YUV), nicht RGB
- **Lösung**: Explizite I420-Caps nach videoconvert setzen

**Finale Pipeline**:
```rust
let pipeline_str = format!(
    "appsrc name=src ! videoconvert ! video/x-raw,format=I420 ! openh264enc bitrate=2000000 ! h264parse ! video/x-h264,stream-format=avc ! mp4mux ! filesink location={}",
    output_path
);
```

### Kamera-Zugriff

**Problem**: Permission denied

**Lösung**:
```bash
sudo usermod -a -G video $USER
```
(Logout/Login erforderlich)

### Fehlende GStreamer-Plugins

**Problem**: Element nicht gefunden (z.B. "no element 'openh264enc'")

**Lösung**: Installiere gstreamer-plugins-bad
```bash
gst-inspect-1.0 openh264enc
```

Verfügbare Encoder überprüfen:
```bash
gst-inspect-1.0 | grep -i enc | grep -i video
```

## Entwicklung

### Code-Stil

- Keine Kommentare im Code (ausschließlich Dokumentation in README)
- Klare, selbsterklärende Funktions- und Variablennamen
- Modulare Struktur mit klaren Verantwortlichkeiten

### Testen

```bash
cargo test
```

### Debugging

GStreamer Debug-Output aktivieren:
```bash
GST_DEBUG=3 ./cam_record_sim
```

### Linting

```bash
cargo clippy
```

## Lizenz

MIT

## Autor

Void @ FasTTube

## Version

0.1.0
