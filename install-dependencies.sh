#!/bin/bash

echo "Installing dependencies for cam_record_sim..."
echo ""

# Detect distribution
if [ -f /etc/fedora-release ]; then
    DISTRO="fedora"
elif [ -f /etc/debian_version ]; then
    DISTRO="debian"
elif [ -f /etc/arch-release ]; then
    DISTRO="arch"
elif [ -f /etc/SuSE-release ] || [ -f /etc/SUSE-release ]; then
    DISTRO="suse"
else
    DISTRO="unknown"
fi

echo "Detected distribution: $DISTRO"
echo ""

case $DISTRO in
    fedora)
        echo "Installing Fedora dependencies..."
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
        ;;

    debian)
        echo "Installing Debian/Ubuntu dependencies..."
        sudo apt-get update
        sudo apt-get install -y \
            libgstreamer1.0-dev \
            libgstreamer-plugins-base1.0-dev \
            gstreamer1.0-plugins-good \
            gstreamer1.0-plugins-bad \
            gstreamer1.0-x \
            libgtk-4-dev \
            libgraphene-1.0-dev \
            libglib2.0-dev \
            libcairo2-dev \
            libpango1.0-dev \
            libgdk-pixbuf2.0-dev

        # Install Rust if not present
        if ! command -v cargo &> /dev/null; then
            echo "Installing Rust..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source $HOME/.cargo/env
        fi
        ;;

    arch)
        echo "Installing Arch Linux dependencies..."
        sudo pacman -Sy --noconfirm \
            rust \
            cargo \
            opencv \
            clang \
            gtk4 \
            graphene \
            glib2 \
            cairo \
            pango \
            gdk-pixbuf2
        ;;

    suse)
        echo "Installing openSUSE dependencies..."
        sudo zypper install -y \
            rust \
            cargo \
            opencv-devel \
            clang \
            gtk4-devel \
            libgraphene-devel \
            glib2-devel \
            cairo-devel \
            pango-devel \
            gdk-pixbuf-devel
        ;;

    *)
        echo "Unknown distribution. Please install manually:"
        echo "  - Rust/Cargo"
        echo "  - OpenCV development files"
        echo "  - GTK4 development files"
        echo "  - Graphene development files"
        echo "  - GLib, Cairo, Pango development files"
        exit 1
        ;;
esac

echo ""
echo "Dependencies installed successfully!"
echo ""
echo "You can now build the project:"
echo "  cargo build --release"
echo ""
echo "Or build all packages:"
echo "  ./build-all.sh"
