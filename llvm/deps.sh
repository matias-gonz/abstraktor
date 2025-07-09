#!/bin/bash
set -e

# Check if we're running as root (common in containers)
if [ "$EUID" -eq 0 ]; then
    SUDO=""
else
    SUDO="sudo"
fi

echo "Installing required dependencies..."
$SUDO apt-get update
$SUDO apt-get install -y libtinfo5 libncurses5 curl xz-utils

# Install Rust if not already present
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    export PATH="$HOME/.cargo/bin:$PATH"
    source "$HOME/.cargo/env"
else
    echo "Rust already installed"
fi

echo "Downloading LLVM 11.0.1 archive..."
curl -L -o clang+llvm-11.0.1-x86_64-linux-gnu-ubuntu-16.04.tar.xz https://github.com/llvm/llvm-project/releases/download/llvmorg-11.0.1/clang+llvm-11.0.1-x86_64-linux-gnu-ubuntu-16.04.tar.xz

echo "Extracting LLVM..."
$SUDO tar -xf clang+llvm-11.0.1-x86_64-linux-gnu-ubuntu-16.04.tar.xz -C /usr/local --strip-components=1

echo "Moving llvm-config to llvm-config-11"
$SUDO mv /usr/local/bin/llvm-config /usr/local/bin/llvm-config-11

echo "Moving clang++ to clang++-11"
$SUDO mv /usr/local/bin/clang++ /usr/local/bin/clang++-11

echo "Adding /usr/local/bin to PATH"
export PATH=/usr/local/bin:$PATH

echo "Checking llvm-config version:"
llvm-config-11 --version
clang++-11 --version

echo "Installing nlohmann-json3-dev"
$SUDO apt-get install nlohmann-json3-dev
