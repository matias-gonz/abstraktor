#!/bin/bash
set -euo pipefail

# Add official LLVM apt repo for older versions
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 12

# Set up environment for LLVM 12
export LLVM_CONFIG=/usr/bin/llvm-config-12
export CC=clang-12
export CXX=clang++-12

# Confirm
echo "Installed LLVM version: $($LLVM_CONFIG --version)"
echo "Using CC=$CC"
echo "Using CXX=$CXX"
