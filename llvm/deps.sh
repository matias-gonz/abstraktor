#!/bin/bash
set -euo pipefail

LLVM_VERSION=12.0.1
LLVM_SHORT=12
INSTALL_DIR=/opt/llvm-$LLVM_SHORT
LLVM_TAR=clang+llvm-$LLVM_VERSION-x86_64-linux-gnu-ubuntu-20.04.tar.xz
LLVM_URL=https://github.com/llvm/llvm-project/releases/download/llvmorg-$LLVM_VERSION/$LLVM_TAR

# Create install dir
mkdir -p "$INSTALL_DIR"

# Download and extract
curl -L "$LLVM_URL" | tar -xJ --strip-components=1 -C "$INSTALL_DIR"

# Export env vars
export PATH="$INSTALL_DIR/bin:$PATH"
export LD_LIBRARY_PATH="$INSTALL_DIR/lib:$LD_LIBRARY_PATH"
export LLVM_CONFIG="$INSTALL_DIR/bin/llvm-config"
export CC=clang
export CXX=clang++

# Confirm setup
clang --version
llvm-config --version
