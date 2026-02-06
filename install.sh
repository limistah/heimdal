#!/bin/bash
#
# Heimdal Installation Script
# 
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/limistah/heimdal/main/install.sh | bash
#   or
#   wget -qO- https://raw.githubusercontent.com/limistah/heimdal/main/install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_header() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE} Heimdal Installation${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            echo "$ID"
        else
            echo "linux"
        fi
    else
        echo "unknown"
    fi
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install Rust if not present
install_rust() {
    if ! command_exists rustc; then
        print_info "Rust not found. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        print_success "Rust installed successfully"
    else
        print_info "Rust already installed"
    fi
}

# Install dependencies
install_dependencies() {
    local os=$(detect_os)
    
    print_info "Installing dependencies for $os..."
    
    case "$os" in
        macos)
            if ! command_exists brew; then
                print_info "Installing Homebrew..."
                /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
            fi
            brew install git || true
            ;;
        ubuntu|debian)
            sudo apt-get update
            sudo apt-get install -y git build-essential curl
            ;;
        fedora|rhel|centos)
            sudo dnf install -y git gcc curl
            ;;
        arch|manjaro)
            sudo pacman -Sy --noconfirm git base-devel curl
            ;;
        *)
            print_warning "Unknown OS: $os. Please install git and build-essential manually."
            ;;
    esac
    
    print_success "Dependencies installed"
}

# Clone and build Heimdal
build_heimdal() {
    local temp_dir=$(mktemp -d)
    
    print_info "Cloning Heimdal repository..."
    git clone https://github.com/limistah/heimdal.git "$temp_dir/heimdal"
    
    print_info "Building Heimdal (this may take a few minutes)..."
    cd "$temp_dir/heimdal"
    cargo build --release
    
    print_info "Installing Heimdal to /usr/local/bin..."
    sudo mv target/release/heimdal /usr/local/bin/
    sudo chmod +x /usr/local/bin/heimdal
    
    # Clean up
    cd -
    rm -rf "$temp_dir"
    
    print_success "Heimdal installed successfully"
}

# Verify installation
verify_installation() {
    if command_exists heimdal; then
        local version=$(heimdal --version | head -n 1)
        print_success "Installation verified: $version"
        return 0
    else
        print_error "Installation failed: heimdal command not found"
        return 1
    fi
}

# Main installation flow
main() {
    print_header
    
    # Check if already installed
    if command_exists heimdal; then
        print_warning "Heimdal is already installed"
        echo ""
        heimdal --version
        echo ""
        read -p "Do you want to reinstall? [y/N] " -n 1 -r
        echo ""
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Installation cancelled"
            exit 0
        fi
    fi
    
    # Install dependencies
    install_dependencies
    
    # Install Rust
    install_rust
    
    # Build and install Heimdal
    build_heimdal
    
    # Verify
    if verify_installation; then
        echo ""
        print_success "Heimdal installation complete!"
        echo ""
        print_info "Get started with:"
        echo "  heimdal init --profile <profile-name> --repo <git-repo-url>"
        echo ""
        print_info "For help, run:"
        echo "  heimdal --help"
        echo ""
    else
        exit 1
    fi
}

# Run main
main
