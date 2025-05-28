#!/bin/bash
set -euo pipefail

# Install dependencies
sudo apt update
sudo apt install -y \
  llvm-12 \
  llvm-12-dev \
  llvm-12-linker-tools \
  llvm-12-runtime \
  llvm-12-tools \
  clang-12

# Configure environment for LLVM 12
export LLVM_CONFIG=/usr/bin/llvm-config-12
export CC=clang-12
export CXX=clang++-12

# Optional: verify setup
echo "Using LLVM config: $($LLVM_CONFIG --version)"
echo "Using CC: $CC"
echo "Using CXX: $CXX"
