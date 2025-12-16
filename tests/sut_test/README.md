# SUT Setup Test Directory

This directory contains test files for the SUT (System Under Test) setup module.

## Structure

- `sample_source/` - A sample source directory with test files
  - `test_file.txt` - Simple text file
  - `config.json` - JSON configuration file
  - `subdir/nested_file.txt` - Nested file in subdirectory

## Usage

These files are used by the test suite to verify that the SUT setup module correctly:
- Validates source paths
- Copies files recursively
- Handles custom and default destination names
- Reports errors appropriately

**Note:** Tests do NOT rebuild Docker images (rebuild flag is always false in tests).

