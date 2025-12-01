#!/bin/bash

set -e

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

# Function to install Rust
install_rust() {
    if ! command -v cargo &> /dev/null; then
        echo "Rust not found. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        echo "Rust installed successfully!"
        echo ""
    else
        echo "Rust is already installed."
        echo ""
    fi
}

case $DISTRO in
    fedora)
        echo "Installing Fedora dependencies..."
        sudo dnf install -y \
            gstreamer1-devel \
            gstreamer1-plugins-base-devel \
            gstreamer1-plugins-good \
            gstreamer1-plugins-bad-free \
            gstreamer1-plugin-openh264 \
            gtk4-devel \
            graphene-devel \
            glib2-devel \
            cairo-devel \
            pango-devel \
            gdk-pixbuf2-devel \
            git \
            cmake

        install_rust

        echo ""
        echo "Installing The Imaging Source camera support (tiscamera)..."
        if [ ! -d "$HOME/tiscamera" ]; then
            git clone https://github.com/TheImagingSource/tiscamera.git "$HOME/tiscamera"
            cd "$HOME/tiscamera"
            mkdir -p build
            cd build
            cmake ..
            make -j$(nproc)
            sudo make install
            sudo ldconfig
            cd ~
            echo "tiscamera installed successfully!"
        else
            echo "tiscamera already exists at $HOME/tiscamera"
        fi

        echo ""
        echo "The Imaging Source DFK 37BUX265 camera support installed!"
        echo "Optional: For better video quality, install gstreamer1-plugins-ugly from RPM Fusion"
        echo "  sudo dnf install gstreamer1-plugins-ugly"
        ;;

    debian)
        echo "Installing Debian/Ubuntu dependencies..."
        sudo apt-get update
        sudo apt-get install -y \
            curl \
            build-essential \
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
            libgdk-pixbuf2.0-dev \
            git \
            cmake \
            pkg-config

        install_rust

        echo ""
        echo "Installing The Imaging Source camera support (tiscamera)..."
        if [ ! -d "$HOME/tiscamera" ]; then
            git clone https://github.com/TheImagingSource/tiscamera.git "$HOME/tiscamera"
            cd "$HOME/tiscamera"
            mkdir -p build
            cd build
            cmake ..
            make -j$(nproc)
            sudo make install
            sudo ldconfig
            cd ~
            echo "tiscamera installed successfully!"
        else
            echo "tiscamera already exists at $HOME/tiscamera"
        fi

        echo ""
        echo "The Imaging Source DFK 37BUX265 camera support installed!"
        ;;

    arch)
        echo "Installing Arch Linux dependencies..."
        sudo pacman -Sy --noconfirm \
            base-devel \
            curl \
            gstreamer \
            gst-plugins-base \
            gst-plugins-good \
            gst-plugins-bad \
            gst-plugin-gtk \
            gtk4 \
            graphene \
            glib2 \
            cairo \
            pango \
            gdk-pixbuf2 \
            git \
            cmake \
            pkg-config

        install_rust

        echo ""
        echo "Installing The Imaging Source camera support (tiscamera)..."
        if [ ! -d "$HOME/tiscamera" ]; then
            git clone https://github.com/TheImagingSource/tiscamera.git "$HOME/tiscamera"
            cd "$HOME/tiscamera"
            mkdir -p build
            cd build
            cmake ..
            make -j$(nproc)
            sudo make install
            sudo ldconfig
            cd ~
            echo "tiscamera installed successfully!"
        else
            echo "tiscamera already exists at $HOME/tiscamera"
        fi

        echo ""
        echo "The Imaging Source DFK 37BUX265 camera support installed!"
        ;;

    suse)
        echo "Installing openSUSE dependencies..."
        sudo zypper install -y \
            curl \
            gcc \
            make \
            gstreamer-devel \
            gstreamer-plugins-base-devel \
            gstreamer-plugins-good \
            gstreamer-plugins-bad \
            gtk4-devel \
            libgraphene-devel \
            glib2-devel \
            cairo-devel \
            pango-devel \
            gdk-pixbuf-devel \
            git \
            cmake \
            pkg-config

        install_rust

        echo ""
        echo "Installing The Imaging Source camera support (tiscamera)..."
        if [ ! -d "$HOME/tiscamera" ]; then
            git clone https://github.com/TheImagingSource/tiscamera.git "$HOME/tiscamera"
            cd "$HOME/tiscamera"
            mkdir -p build
            cd build
            cmake ..
            make -j$(nproc)
            sudo make install
            sudo ldconfig
            cd ~
            echo "tiscamera installed successfully!"
        else
            echo "tiscamera already exists at $HOME/tiscamera"
        fi

        echo ""
        echo "The Imaging Source DFK 37BUX265 camera support installed!"
        ;;

    *)
        echo "Unknown distribution!"
        echo ""
        echo "Attempting to install Rust..."
        install_rust
        echo ""
        echo "Please install the following dependencies manually:"
        echo "  - GStreamer development files"
        echo "  - GStreamer plugins: base, good, bad"
        echo "  - GTK4 development files"
        echo "  - Graphene development files"
        echo "  - GLib, Cairo, Pango development files"
        echo "  - pkg-config"
        exit 1
        ;;
esac

echo ""
echo "Dependencies installed successfully!"
echo ""
echo "Note: If this is your first time installing Rust, you need to reload your shell environment:"
echo "  source \$HOME/.cargo/env"
echo ""
echo "You can now build the project:"
echo "  cargo build --release"
echo ""
echo "Or run directly:"
echo "  cargo run --release"
