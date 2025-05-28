#!/bin/bash
set -euo pipefail

# Add jammy repo with trusted=yes to skip Release file checks
echo "deb [trusted=yes] http://apt.llvm.org/jammy/ llvm-toolchain-jammy-12 main" | sudo tee /etc/apt/sources.list.d/llvm12.list

# Update package index
sudo apt-get update

# Install LLVM 12 and Clang 12
sudo apt-get install -y llvm-12 llvm-12-dev llvm-12-tools clang-12

# Set environment variables for builds
export LLVM_CONFIG=/usr/bin/llvm-config-12
export CC=clang-12
export CXX=clang++-12

echo "✅ LLVM 12 installed and configured"
