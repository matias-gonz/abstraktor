# Abstraktor Project Context

## Overview
Abstraktor is a Rust-based tool for validating distributed protocol implementations through enabling-preserving abstractions (EPAs). It instruments code, performs fuzzing with Mallory, and generates abstractions to find business logic issues.

## Key Commands

### Build & Test
```bash
cargo build
cargo test
cargo build --release
```

### Linting & Type Checking
```bash
cargo clippy
cargo check
```

### Running the CLI
```bash
cargo run -- [command]
```

## Project Structure
- `src/main.rs` - Entry point, CLI handling
- `src/commands/` - Core commands:
  - `get_targets.rs` - Extract instrumentation targets from source
  - `instrument.rs` - Instrument binaries using LLVM
  - `llvm.rs` - LLVM-related operations
- `llvm/` - LLVM instrumentation components (AFL-based)
- `tests/` - Test files including e2e tests

## Workflow Components
1. **prepare-binary** - Instruments source code based on annotations
2. **run-mallory** - Executes instrumented binary under Mallory fuzzer
3. **generate-epas** - Creates EPAs from collected runtime events

## Important Files
- `BB2ID.txt` - Basic block to ID mappings
- `build.rs` - Build script for LLVM components
- Test example: `tests/instrument_test/` contains a complete test case

## Dependencies
- Rust toolchain
- LLVM components (built via `llvm/deps.sh`)
- Mallory fuzzing tool (external dependency)

## Testing Approach
- Unit tests: `cargo test`
- E2E test documentation: `tests/basic-e2e-test.md`
- Test programs in `tests/instrument_test/`

## Git Workflow

### IMPORTANT: Before editing any code
1. **Always** checkout to main branch first:
   ```bash
   git checkout main
   ```
2. Pull latest changes:
   ```bash
   git pull
   ```
3. Create a new feature branch:
   ```bash
   git checkout -b feature/[descriptive-branch-name]
   ```

### Branch Naming Convention
- `feature/` - for new features
- `fix/` - for bug fixes
- `refactor/` - for code refactoring
- `docs/` - for documentation updates

### After Making Changes
1. Run tests: `cargo test`
2. Run linter: `cargo clippy`
3. Commit with clear messages
4. Push to remote: `git push -u origin [branch-name]`

### Commit and PR Guidelines
- **DO NOT** include Claude as co-author in commits
- **DO NOT** mention Claude or AI assistance in PRs
- Write commit messages as if you (the developer) made the changes