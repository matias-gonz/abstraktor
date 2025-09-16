#!/bin/sh

echo "Running install.sh from: $(pwd)"
echo "CC=$CC"
echo "TARGETS_FILE=$TARGETS_FILE"

make clean
make all

echo "Build complete: $(pwd)/fsm_app"

