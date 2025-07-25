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
$SUDO apt-get install -y libtinfo5 libncurses5 curl xz-utils build-essential

echo "Downloading LLVM 11.0.1 archive..."
curl -L -o clang+llvm-11.0.1-x86_64-linux-gnu-ubuntu-16.04.tar.xz https://github.com/llvm/llvm-project/releases/download/llvmorg-11.0.1/clang+llvm-11.0.1-x86_64-linux-gnu-ubuntu-16.04.tar.xz

echo "Extracting LLVM..."
$SUDO tar -xf clang+llvm-11.0.1-x86_64-linux-gnu-ubuntu-16.04.tar.xz -C /usr/local --strip-components=1

echo "Moving llvm-config to llvm-config-11"
$SUDO ln -s /usr/bin/llvm-config-11 /usr/bin/llvm-config

echo "Moving clang++ to clang++-11"
$SUDO ln -s /usr/bin/clang-11 /usr/bin/clang && ln -s /usr/bin/clang++-11 /usr/bin/clang++

echo "Adding /usr/local/bin to PATH"
export PATH=/usr/local/bin:$PATH

echo "Checking llvm-config version:"
llvm-config --version
clang++ --version

echo "Installing nlohmann-json3-dev"
$SUDO apt-get install nlohmann-json3-dev
