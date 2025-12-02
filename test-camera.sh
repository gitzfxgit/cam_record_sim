#!/bin/bash

# Camera Test Script with GStreamer
# Tests cameras directly with configurable parameters

set -e

# Default values
DEVICE="/dev/video0"
WIDTH=640
HEIGHT=480
FPS=30
BAYER_FORMAT="rggb"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Help function
show_help() {
    echo "Camera Test Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -d, --device DEVICE    Video device (default: /dev/video0)"
    echo "  -i, --index INDEX      Video device by index (e.g., 0 for /dev/video0)"
    echo "  -w, --width WIDTH      Width in pixels (default: 640)"
    echo "  -h, --height HEIGHT    Height in pixels (default: 480)"
    echo "  -f, --fps FPS          Frames per second (default: 30)"
    echo "  -b, --bayer FORMAT     Bayer format: rggb, bggr, grbg, gbrg (default: rggb)"
    echo "  -l, --list             List all available cameras"
    echo "  -t, --test             Test without Bayer conversion (for standard webcams)"
    echo "  --help                 Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 -i 0                          # Test /dev/video0 with defaults"
    echo "  $0 -i 2                          # Test /dev/video2 (DFK camera)"
    echo "  $0 -i 2 -w 1920 -h 1080 -f 60   # Test DFK at 1080p 60fps"
    echo "  $0 -d /dev/video2 -w 2048 -h 1536 -f 30  # Test DFK at max resolution"
    echo "  $0 -t -i 0                       # Test standard webcam"
    echo ""
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--device)
            DEVICE="$2"
            shift 2
            ;;
        -i|--index)
            DEVICE="/dev/video$2"
            shift 2
            ;;
        -w|--width)
            WIDTH="$2"
            shift 2
            ;;
        -h|--height)
            HEIGHT="$2"
            shift 2
            ;;
        -f|--fps)
            FPS="$2"
            shift 2
            ;;
        -b|--bayer)
            BAYER_FORMAT="$2"
            shift 2
            ;;
        -l|--list)
            echo "Available cameras:"
            echo ""
            if command -v v4l2-ctl &> /dev/null; then
                v4l2-ctl --list-devices
            else
                ls -l /dev/video* 2>/dev/null || echo "No video devices found"
            fi
            exit 0
            ;;
        -t|--test)
            TEST_MODE=1
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

echo "========================================"
echo "Camera Test Script"
echo "========================================"
echo ""

# Check if device exists
if [ ! -e "$DEVICE" ]; then
    echo -e "${RED}✗ Error: Device $DEVICE does not exist!${NC}"
    echo ""
    echo "Available devices:"
    ls -l /dev/video* 2>/dev/null || echo "No video devices found"
    exit 1
fi

# Check if device is readable
if [ ! -r "$DEVICE" ]; then
    echo -e "${RED}✗ Error: Cannot read $DEVICE (permission denied)${NC}"
    echo ""
    echo "Solutions:"
    echo "  1. Add yourself to video group: sudo usermod -a -G video \$USER"
    echo "  2. Log out and back in"
    echo "  3. Or temporarily: sudo chmod a+rw $DEVICE"
    exit 1
fi

echo "Device:     $DEVICE"
echo "Resolution: ${WIDTH}x${HEIGHT}"
echo "FPS:        ${FPS}"
echo ""

# Get device info
if command -v v4l2-ctl &> /dev/null; then
    echo "Camera Info:"
    CARD_TYPE=$(v4l2-ctl --device=$DEVICE --info 2>/dev/null | grep "Card type" | cut -d: -f2 | xargs)
    if [ -n "$CARD_TYPE" ]; then
        echo "  Name: $CARD_TYPE"
    fi

    # Check supported formats
    echo ""
    echo "Supported formats:"
    FORMATS=$(v4l2-ctl --device=$DEVICE --list-formats 2>/dev/null)

    if echo "$FORMATS" | grep -q "Bayer\|RGGB\|RG16"; then
        echo -e "  ${GREEN}✓ Bayer format detected (Industrial camera)${NC}"
        BAYER_CAMERA=1
        echo "  Using Bayer format: $BAYER_FORMAT"
    else
        echo -e "  ${YELLOW}Standard RGB/YUV camera (not Bayer)${NC}"
        BAYER_CAMERA=0
    fi
    echo ""
fi

# Build GStreamer pipeline
if [ -n "$TEST_MODE" ] || [ "$BAYER_CAMERA" != "1" ]; then
    # Standard camera (no Bayer conversion)
    echo "Testing standard camera pipeline..."
    echo ""
    PIPELINE="v4l2src device=$DEVICE ! \
        video/x-raw,width=$WIDTH,height=$HEIGHT,framerate=$FPS/1 ! \
        videoconvert ! \
        autovideosink"
else
    # Bayer camera with conversion
    echo "Testing Bayer camera pipeline..."
    echo "Bayer format: $BAYER_FORMAT"
    echo ""
    PIPELINE="v4l2src device=$DEVICE ! \
        video/x-bayer,format=$BAYER_FORMAT,width=$WIDTH,height=$HEIGHT,framerate=$FPS/1 ! \
        bayer2rgb ! \
        videoconvert ! \
        autovideosink"
fi

echo "GStreamer Pipeline:"
echo "-------------------"
echo "$PIPELINE" | sed 's/ ! /\n  ! /g'
echo ""
echo "Starting video preview..."
echo "Press Ctrl+C to stop"
echo ""

# Run GStreamer
if gst-launch-1.0 $PIPELINE 2>&1; then
    echo ""
    echo -e "${GREEN}✓ Camera test successful!${NC}"
else
    EXIT_CODE=$?
    echo ""
    echo -e "${RED}✗ Camera test failed (exit code: $EXIT_CODE)${NC}"
    echo ""
    echo "Common issues:"
    echo "  - Wrong resolution/fps combination"
    echo "  - Camera is busy (used by another program)"
    echo "  - Wrong Bayer format (try -b bggr, -b grbg, or -b gbrg)"
    echo "  - Missing GStreamer plugins (install gstreamer1.0-plugins-bad)"
    echo ""
    echo "Check supported formats and resolutions:"
    echo "  v4l2-ctl --device=$DEVICE --list-formats-ext"
    exit $EXIT_CODE
fi
