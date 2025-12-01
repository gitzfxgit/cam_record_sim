# Camera Record Simulator

Dual-camera recording and simulation system with GTK4 GUI. Records from real cameras and plays back recordings in an endless loop as virtual cameras - perfect for stereo vision development and testing without hardware.

## Overview

This program provides two main functions:

1. **Recording Tab**: Records from 1-2 real cameras and saves videos to a configurable folder
2. **Simulation Tab**: Loads recorded videos and plays them in a loop as if 2 real cameras were connected

## Features

- **Dual-Camera Recording**: Simultaneous recording from two cameras with live preview
- **Video Simulation**: Play back saved videos as virtual cameras (endless loop)
- **Live Preview**: Real-time display of both camera feeds during recording and simulation
- **Flexible Configuration**: Adjustable FPS, resolution, duration, and output folder
- **Metadata Export**: Automatic JSON metadata for each recording
- **GTK4 GUI**: Modern, intuitive user interface
- **CLI Support**: Complete command-line interface

## System Requirements

### Dependencies

**GStreamer** (for video encoding/decoding):
- gstreamer
- gstreamer-plugins-base
- gstreamer-plugins-good
- gstreamer-plugins-bad (contains openh264enc for H.264 encoding)

**GTK4** (for GUI):
- gtk4
- glib

**Rust** (for compilation):
- Rust 2024 Edition
- cargo

### Installing Dependencies

The installation script automatically detects your Linux distribution and installs all required dependencies, including Rust if not already installed:

```bash
./install-dependencies.sh
```

Supported distributions:
- **Fedora / Red Hat / CentOS**
- **Ubuntu / Debian**
- **Arch Linux**
- **openSUSE**

## Building

```bash
cargo build --release
./target/release/cam_record_sim
```

Or run directly:
```bash
cargo run --release
```

## Usage

### GUI Mode (Default)

Start the program without parameters:
```bash
./cam_record_sim
```

#### Recording Tab

1. **Camera Selection**:
   - Choose "1 Camera" or "2 Cameras"
   - Specify camera IDs (usually 0 and 1)

2. **Settings**:
   - FPS: Frames per second (default: 30)
   - Duration: Recording duration in seconds (default: 10)
   - Output: Target folder for videos (default: recordings)

3. **Recording**:
   - Click "Start Recording"
   - Live preview shows both camera feeds
   - Videos are saved with timestamp: `camera_0__YYYYMMDD_HHMMSS.mp4`

#### Simulation Tab

1. **Load Videos**:
   - Enter the folder containing recordings (e.g., "recordings")
   - Click "Load"
   - The first two MP4 files will be loaded:
     - First file → Virtual Camera 0 (Left)
     - Second file → Virtual Camera 1 (Right)

2. **Start Simulation**:
   - Click "Start Simulation"
   - Both videos play in an endless loop
   - Live preview shows both video feeds
   - Videos can now be used by other applications as camera input

3. **Stop Simulation**:
   - Click "Stop Simulation"

### CLI Mode

All functions are also available via command line:

#### List Cameras
```bash
./cam_record_sim list-cameras
```

#### Record from Real Camera
```bash
./cam_record_sim record \
    --camera 0 \
    --output recordings \
    --fps 30.0 \
    --duration 60
```

#### Record from Virtual Test Camera
```bash
./cam_record_sim sim-record \
    --camera 0 \
    --output recordings \
    --fps 30.0 \
    --duration 10
```

#### Test Virtual Cameras
```bash
./cam_record_sim test-virtual --duration 5
```

#### List Recordings
```bash
./cam_record_sim list-recordings --dir recordings
```

#### Play Recording
```bash
./cam_record_sim play \
    --file camera_0__20241130_120000.mp4 \
    --dir recordings
```

## Architecture

### Modules

#### `main.rs`
- CLI argument parsing with clap
- Main entry point
- Routing between GUI and CLI modes

#### `gui.rs`
- GTK4 user interface
- Three tabs: Recording, Simulation, Playback
- Event handling and UI updates
- Live preview rendering

#### `camera.rs`
- Real camera management
- Based on nokhwa library
- Frame capturing in RGB format

#### `virtual_camera.rs`
- Generates test patterns (moving color bars)
- Useful for testing without real hardware
- Different colors per camera ID

#### `playback_camera.rs`
- Plays videos as virtual cameras
- GStreamer-based video decoding
- Supports loop mode for endless playback
- `StereoPlaybackSystem`: Manages left and right cameras

#### `recorder.rs`
- Video recording with GStreamer
- H.264 encoding (openh264enc)
- MP4 container
- Frame-by-frame writing
- Metadata export

#### `dual_recorder.rs`
- Coordinates recording from multiple cameras
- Supports:
  - `CameraSource::Single`: One camera
  - `CameraSource::Dual`: Two real cameras
  - `CameraSource::Virtual`: Two test cameras
- Thread-based asynchronous recording

#### `player.rs`
- Video playback
- GStreamer-based
- Supports MP4, AVI, MKV

### GStreamer Pipelines

#### Recording Pipeline
```
appsrc → videoconvert → video/x-raw,format=I420 → openh264enc → h264parse → mp4mux → filesink
```

**Parameters**:
- Input: RGB frames from camera
- `videoconvert`: Converts RGB to I420 (YUV)
- `video/x-raw,format=I420`: Explicit I420 caps (required for openh264enc)
- `openh264enc bitrate=2000000`: 2 Mbps bitrate for good quality
- `h264parse`: Converts byte-stream to avc format for mp4mux
- `video/x-h264,stream-format=avc`: AVC format for MP4 container
- Output: H.264 in MP4 container

#### Playback Pipeline
```
filesrc → decodebin → videoconvert → appsink
```

**Parameters**:
- `decodebin`: Automatic format detection
- `video/x-raw,format=RGB`: RGB output for frame extraction

#### Display Pipeline
```
filesrc → decodebin → videoconvert → autovideosink
```

## Technical Details

### Video Format

**Encoding**:
- Codec: H.264 (openh264enc)
- Container: MP4
- Pixel Format: RGB (input), I420 (encoding)
- Resolution: 640x480 (default)
- FPS: 30 (configurable)
- Bitrate: 2 Mbps (default)

**Metadata** (JSON):
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

### Threading Model

- **Main Thread**: GTK event loop
- **Recording Thread**: Asynchronous camera capture and video encoding
- **UI Update**: glib::timeout_add_local for live preview (30 FPS)

### Frame Flow

#### Recording
```
Camera → get_frame() → VideoRecorder → x264enc → MP4 file
         ↓
    Live Preview (GUI)
```

#### Simulation
```
MP4 file → PlaybackCamera → get_frame() → Live Preview (GUI)
                                         ↓
                                    Other Application
```

## Use Cases

### 1. Stereo Vision Development

**Problem**: Hardware setup (two cameras) is impractical during development

**Solution**:
1. One-time recording from both cameras in Recording tab
2. Simulation of cameras in Simulation tab
3. Development with reproducible video inputs
4. No hardware needed anymore

### 2. Continuous Integration

**Problem**: CI servers don't have camera hardware

**Solution**:
1. Store recordings in repository
2. Tests use playback cameras instead of real hardware
3. Reproducible, deterministic tests

### 3. Offline Development

**Problem**: Camera hardware not always available

**Solution**:
1. Create recordings once with hardware
2. Continue development later without hardware
3. Same code base, simulated inputs

## Troubleshooting

### GStreamer Pipeline Errors

**Error 1**: `no element "x264enc"`
- **Cause**: x264 plugin not installed (requires gst-plugins-ugly)
- **Solution**: Use openh264enc (included in gst-plugins-bad)

**Error 2**: `could not link openh264enc0 to mp4mux0`
- **Cause**: openh264enc outputs byte-stream, mp4mux needs avc format
- **Solution**: Insert h264parse between openh264enc and mp4mux

**Error 3**: `[OpenH264] Error:CWelsH264SVCEncoder::EncodeFrame(), cmInitParaError`
- **Cause**: openh264enc only accepts I420 (YUV), not RGB
- **Solution**: Set explicit I420 caps after videoconvert

**Final Pipeline**:
```rust
let pipeline_str = format!(
    "appsrc name=src ! videoconvert ! video/x-raw,format=I420 ! openh264enc bitrate=2000000 ! h264parse ! video/x-h264,stream-format=avc ! mp4mux ! filesink location={}",
    output_path
);
```

### Camera Access

**Problem**: Permission denied

**Solution**:
```bash
sudo usermod -a -G video $USER
```
(Logout/login required)

### Missing GStreamer Plugins

**Problem**: Element not found (e.g., "no element 'openh264enc'")

**Solution**: Install gstreamer-plugins-bad
```bash
gst-inspect-1.0 openh264enc
```

Check available encoders:
```bash
gst-inspect-1.0 | grep -i enc | grep -i video
```

## Development

### Code Style

- No comments in code (documentation only in README)
- Clear, self-explanatory function and variable names
- Modular structure with clear responsibilities

### Testing

```bash
cargo test
```

### Debugging

Enable GStreamer debug output:
```bash
GST_DEBUG=3 ./cam_record_sim
```

### Linting

```bash
cargo clippy
```

## License

MIT

## Author

FastTube Team

## Version

0.1.0
