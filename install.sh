#!/bin/bash
set -e

# nm-remover installer
# Usage: curl -fsSL https://raw.githubusercontent.com/supostat/node-modules-remover/main/install.sh | bash

REPO="supostat/node-modules-remover"
BINARY_NAME="nm-remover"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════╗"
    echo "║         nm-remover installer          ║"
    echo "║   Node Modules Cleanup Tool           ║"
    echo "╚═══════════════════════════════════════╝"
    echo -e "${NC}"
}

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

detect_os() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)
            OS="linux"
            ;;
        Darwin*)
            OS="darwin"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            OS="windows"
            ;;
        *)
            error "Unsupported operating system: $OS"
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        arm64|aarch64)
            ARCH="aarch64"
            ;;
        *)
            error "Unsupported architecture: $ARCH"
            ;;
    esac

    info "Detected OS: $OS, Architecture: $ARCH"
}

get_latest_version() {
    info "Fetching latest version..."

    LATEST_VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

    if [ -z "$LATEST_VERSION" ]; then
        error "Failed to fetch latest version. Please check your internet connection."
    fi

    info "Latest version: $LATEST_VERSION"
}

download_binary() {
    local version="$1"
    local os="$2"
    local arch="$3"

    # Construct download URL
    local filename="${BINARY_NAME}-${os}-${arch}"
    if [ "$os" = "windows" ]; then
        filename="${filename}.exe"
    fi

    local url="https://github.com/${REPO}/releases/download/${version}/${filename}"

    info "Downloading from: $url"

    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT

    # Download binary
    if ! curl -fsSL "$url" -o "$TMP_DIR/$BINARY_NAME"; then
        error "Failed to download binary. The release might not exist for your platform."
    fi

    # Make executable
    chmod +x "$TMP_DIR/$BINARY_NAME"

    success "Downloaded successfully!"
}

install_binary() {
    info "Installing to: $INSTALL_DIR"

    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"

    # Move binary to install directory
    mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"

    success "Installed $BINARY_NAME to $INSTALL_DIR"
}

check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "$INSTALL_DIR is not in your PATH"
        echo ""
        echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo -e "${YELLOW}  export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
        echo ""
        echo "Then reload your shell or run:"
        echo ""
        echo -e "${YELLOW}  source ~/.bashrc  # or ~/.zshrc${NC}"
        echo ""
    fi
}

verify_installation() {
    if [ -x "$INSTALL_DIR/$BINARY_NAME" ]; then
        success "Installation verified!"
        echo ""
        echo "Run '${BINARY_NAME} --help' to get started"
        echo ""
        "$INSTALL_DIR/$BINARY_NAME" --version 2>/dev/null || true
    else
        error "Installation verification failed"
    fi
}

build_from_source() {
    info "Building from source..."

    # Check if cargo is installed
    if ! command -v cargo &> /dev/null; then
        error "Cargo is not installed. Please install Rust first: https://rustup.rs"
    fi

    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT

    # Clone repository
    info "Cloning repository..."
    git clone --depth 1 "https://github.com/${REPO}.git" "$TMP_DIR/repo"

    # Build
    info "Building..."
    cd "$TMP_DIR/repo"
    cargo build --release

    # Copy binary
    cp "target/release/$BINARY_NAME" "$TMP_DIR/$BINARY_NAME"

    success "Build completed!"
}

main() {
    print_banner
    detect_os

    # Check for --source flag
    if [ "${1:-}" = "--source" ]; then
        build_from_source
    else
        get_latest_version
        download_binary "$LATEST_VERSION" "$OS" "$ARCH"
    fi

    install_binary
    check_path
    verify_installation

    echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║     Installation complete! 🎉         ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
}

main "$@"
