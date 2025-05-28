#!/bin/bash
set -e

echo "Downloading LLVM 12.0.1 archive..."
curl -L -o clang+llvm-12.0.1-x86_64-linux-gnu-ubuntu-16.04.tar.xz https://github.com/llvm/llvm-project/releases/download/llvmorg-12.0.1/clang+llvm-12.0.1-x86_64-linux-gnu-ubuntu-16.04.tar.xz

echo "Extracting LLVM..."
sudo tar -xf clang+llvm-12.0.1-x86_64-linux-gnu-ubuntu-16.04.tar.xz -C /usr/local --strip-components=1

echo "Adding /usr/local/bin to PATH"
export PATH=/usr/local/bin:$PATH

echo "Checking llvm-config version:"
llvm-config --version
