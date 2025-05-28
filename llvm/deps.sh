#!/bin/bash
set -euo pipefail

LLVM_VERSION=12.0.1
INSTALL_DIR=/opt/llvm-$LLVM_VERSION
LLVM_TAR=clang+llvm-$LLVM_VERSION-x86_64-linux-gnu-ubuntu-20.04.tar.xz
LLVM_URL=https://github.com/llvm/llvm-project/releases/download/llvmorg-$LLVM_VERSION/$LLVM_TAR

# Download and extract LLVM
mkdir -p "$INSTALL_DIR"
curl -L "$LLVM_URL" | tar -xJ --strip-components=1 -C "$INSTALL_DIR"

# Export paths
export PATH="$INSTALL_DIR/bin:$PATH"
export LD_LIBRARY_PATH="$INSTALL_DIR/lib:$LD_LIBRARY_PATH"
export LLVM_CONFIG="$INSTALL_DIR/bin/llvm-config"
export CC=clang
export CXX=clang++

# Print version info
clang --version
llvm-config --version
