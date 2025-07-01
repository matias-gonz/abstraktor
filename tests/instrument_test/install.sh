#!/bin/sh

echo "Running install.sh from: $(pwd)"
echo "CC=$CC"
echo "TARGETS_FILE=$TARGETS_FILE"

# Clean previous build
make clean

# Build with instrumentation
make all

echo "Instrumentation complete!"