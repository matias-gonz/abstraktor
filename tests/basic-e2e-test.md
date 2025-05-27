# Basic End-to-End Test

This document explains how to run the basic end-to-end test for the project.

## Prerequisites

Make sure you have the following installed:
- LLVM development tools
- Rust and Cargo
- Make

## Test Steps

### 1. Compile the LLVM Components

First, compile the LLVM instrumentation runtime:

```bash
cd llvm
make
```

This will build the necessary LLVM components including the runtime library.

### 2. Run the Instrumentation Test

Navigate back to the project root and run the instrumentation test:

```bash
cargo run instrument --path tests/instrument_test
```

This command will:
- Instrument the test code in `tests/instrument_test`
- Generate the necessary binaries with instrumentation

## Current Status

✅ **Implemented**: LLVM compilation and instrumentation test
🔄 **Next Steps**: Compile and run mallory with the generated binaries

## Next Steps

The following steps are planned for future implementation:

1. **Compile Mallory**: Build the mallory fuzzer with the instrumented binaries
2. **Run E2E Test**: Execute the complete end-to-end test including Mallory fuzzer execution
s
## Notes

- The current implementation covers the instrumentation phase
- The LLVM runtime provides coverage feedback for AFL-style fuzzing
- The instrumented binaries will be used by mallory for guided fuzzing
