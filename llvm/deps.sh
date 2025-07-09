#!/bin/bash
set -e

echo "Installing required dependencies..."
sudo apt-get update
sudo apt-get install -y libtinfo5 libncurses5

echo "Downloading LLVM 11.0.1 archive..."
curl -L -o clang+llvm-11.0.1-x86_64-linux-gnu-ubuntu-16.04.tar.xz https://github.com/llvm/llvm-project/releases/download/llvmorg-11.0.1/clang+llvm-11.0.1-x86_64-linux-gnu-ubuntu-16.04.tar.xz

echo "Extracting LLVM..."
sudo tar -xf clang+llvm-11.0.1-x86_64-linux-gnu-ubuntu-16.04.tar.xz -C /usr/local --strip-components=1

echo "Moving llvm-config to llvm-config-11"
sudo mv /usr/local/bin/llvm-config /usr/local/bin/llvm-config-11

echo "Moving clang++ to clang++-11"
sudo mv /usr/local/bin/clang++ /usr/local/bin/clang++-11

echo "Adding /usr/local/bin to PATH"
export PATH=/usr/local/bin:$PATH

echo "Checking llvm-config version:"
llvm-config-11 --version
clang++-11 --version

echo "Installing nlohmann-json3-dev"
sudo apt-get install nlohmann-json3-dev
