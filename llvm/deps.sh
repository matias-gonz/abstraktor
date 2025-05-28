#!/bin/bash
set -euo pipefail

# Add LLVM GPG key
wget -qO - https://apt.llvm.org/llvm-snapshot.gpg.key | sudo apt-key add -

# Force jammy repo on noble
echo "deb http://apt.llvm.org/jammy/ llvm-toolchain-jammy-12 main" | sudo tee /etc/apt/sources.list.d/llvm12.list

# Update and install
sudo apt update
sudo apt install -y llvm-12 llvm-12-dev llvm-12-tools clang-12

# Set environment for builds
export LLVM_CONFIG=/usr/bin/llvm-config-12
export CC=clang-12
export CXX=clang++-12

echo "LLVM 12 setup complete"
