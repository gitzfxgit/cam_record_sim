# Camera Record Simulator

Dual-camera recording and simulation system with GTK4 GUI. Records from real cameras and plays back recordings in an endless loop as virtual cameras - perfect for stereo vision development and testing without hardware.

## Overview

This program provides two main functions:

1. **Recording Tab**: Records from 1-2 real cameras and saves videos to a configurable folder

2. **Simulation Tab**: Loads recorded videos and plays them in a loop as if 2 real cameras were connected

## Features

- **Dual-Camera Recording**: Simultaneous recording from two cameras with live preview
- **Industrial Camera Support**: Full support for The Imaging Source DFK 37BUX265 (USB 3.1, Sony IMX265 sensor, 3.1 MP, up to 60 fps)
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

**Industrial Camera Support**:

- tiscamera (The Imaging Source camera SDK)
- GStreamer plugins for USB 3.0/3.1 cameras

### Installing Dependencies

The installation script automatically detects your Linux distribution and installs all required dependencies, including Rust if not already installed:

```bash
./install-dependencies.sh
```

This will install:

- All GStreamer libraries and plugins
- GTK4 and dependencies
- Rust toolchain (if not present)
- **The Imaging Source tiscamera SDK** for DFK 37BUX265 support

Supported distributions:

- **Fedora / Red Hat / CentOS**
- **Ubuntu / Debian**
- **Arch Linux**
- **openSUSE**

### Supported Cameras

**Standard USB Cameras**:

- Any USB webcam or camera supported by Video4Linux (V4L2)
- Typical consumer webcams (Logitech, etc.)

**Industrial Cameras**:

- **The Imaging Source DFK 37BUX265**
  - USB 3.1 interface
  - 1/1.8" Sony CMOS Pregius IMX265 sensor
  - 2048×1536 (3.1 MP) resolution
  - Up to 60 fps
  - Global shutter
  - Trigger and I/O inputs
  - Compact: 42×42×25 mm

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

- Real camera management with dual backend support
- Nokhwa backend for standard USB cameras
- GStreamer backend for industrial Bayer cameras (The Imaging Source)
- Automatic backend selection based on camera capabilities
- Frame capturing in RGB format

#### `gst_camera.rs`

- GStreamer-based camera capture for Bayer format cameras
- Supports The Imaging Source DFK 37BUX265 and similar industrial cameras
- Automatic Bayer-to-RGB conversion using GStreamer bayer2rgb element
- Pipeline: v4l2src → video/x-bayer → bayer2rgb → videoconvert → RGB output
- Configurable resolution and framerate

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

#### Camera Capture Pipeline (Bayer Cameras)

```
v4l2src device=/dev/videoX → video/x-bayer,format=rggb → bayer2rgb → videoconvert → video/x-raw,format=RGB → appsink
```

**Parameters**:

- Input: Raw Bayer RGGB data from The Imaging Source DFK 37BUX265
- `v4l2src`: Direct Video4Linux2 access
- `video/x-bayer,format=rggb`: Bayer RGGB format specification
- `bayer2rgb`: Demosaicing (Bayer to RGB conversion)
- `videoconvert`: Format conversion
- `appsink`: Frame extraction for application use
- Supports multiple resolutions:
  - 640×480 @ 30-370 fps
  - 1920×1080 @ 30-90 fps
  - 2048×1536 @ 15-60 fps

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

## Industrial Camera Setup (The Imaging Source DFK 37BUX265)

### Camera Detection

The application automatically detects Bayer format cameras and uses the GStreamer backend:

```bash
./cam_record_sim list-cameras
```

Expected output:

```
Found cameras:
  - DFK 37BUX265 (/dev/video2)
  - DFK 37BUX265 (/dev/video3)
```

### Checking Camera Capabilities

To verify the camera's supported formats and resolutions:

```bash
v4l2-ctl --device /dev/video2 --list-formats-ext
```

Expected output for DFK 37BUX265:

```
[0]: 'RGGB' (8-bit Bayer RGRG/GBGB)
    Size: Discrete 640x480
        Interval: Discrete 0.003s (370.000 fps)
        ...
    Size: Discrete 1920x1080
        Interval: Discrete 0.017s (60.000 fps)
        ...
    Size: Discrete 2048x1536
        Interval: Discrete 0.017s (60.000 fps)
        ...
```

### Testing Camera with GStreamer

Test the camera directly with GStreamer:

```bash
# Test at 640x480
gst-launch-1.0 v4l2src device=/dev/video2 ! \
  video/x-bayer,format=rggb,width=640,height=480,framerate=30/1 ! \
  bayer2rgb ! videoconvert ! autovideosink

# Test at 1920x1080
gst-launch-1.0 v4l2src device=/dev/video2 ! \
  video/x-bayer,format=rggb,width=1920,height=1080,framerate=60/1 ! \
  bayer2rgb ! videoconvert ! autovideosink
```

### Recording from DFK 37BUX265

```bash
# Record at 640x480 @ 30fps
./cam_record_sim record --camera 2 --duration 10 --fps 30.0

# The application automatically:
# 1. Detects the Bayer format
# 2. Uses GStreamer backend
# 3. Converts Bayer to RGB
# 4. Records to MP4
```

### Required GStreamer Plugins

The DFK 37BUX265 requires the `bayer2rgb` element from gstreamer-plugins-bad:

```bash
# Check if bayer2rgb is available
gst-inspect-1.0 bayer2rgb

# If missing, install gstreamer-plugins-bad
# Fedora/RHEL:
sudo dnf install gstreamer1-plugins-bad

# Ubuntu/Debian:
sudo apt install gstreamer1.0-plugins-bad
```

### Common Issues

**Camera not detected**:

- Check USB connection (USB 3.0/3.1 port required)
- Verify camera appears in /dev: `ls -l /dev/video*`
- Check permissions: `sudo usermod -a -G video $USER` (logout required)

**Low framerate**:

- Ensure USB 3.0/3.1 connection (not USB 2.0)
- Check USB bandwidth: `lsusb -v | grep -i bandwidth`
- Reduce resolution or framerate

**No video output**:

- Verify bayer2rgb plugin: `gst-inspect-1.0 bayer2rgb`
- Test with gst-launch-1.0 (see commands above)
- Check GStreamer debug: `GST_DEBUG=3 ./cam_record_sim`

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

Void @ FaSTTUBe

## Version

0.1.0
