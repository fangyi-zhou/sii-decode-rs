# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust library for decoding SII files (Unit serialized files) used in SCS Software games (Euro Truck Simulator 2, American Truck Simulator). The library is compiled to WebAssembly for use in a React-based web interface at https://sii-decode.github.io/

## Build Commands

### Rust Library
```bash
cargo build                    # Basic build
cargo build --all-features     # Build with WASM support
```

### WebAssembly
```bash
cargo install wasm-pack        # One-time install
wasm-pack build --all-features # Outputs to ./pkg directory
```

### Web Interface
```bash
cd web
yarn                           # Install dependencies
yarn dev                       # Start dev server (Vite)
yarn build                     # Production build
```

### CLI Binary
```bash
cargo run -- [FILE_TO_DECODE]  # Prints decoded output to stdout
```

## Testing

### Rust
```bash
cargo test                                    # Run all tests
cargo test --all-features                     # Include WASM tests
cargo test test_name -- --exact --nocapture   # Single test
```

### WebAssembly
```bash
wasm-pack test --chrome --headless --all-features
```

### Web Interface
```bash
cd web
yarn playwright install        # One-time browser install
yarn test                      # Run vitest (watch mode)
yarn test App.test             # Run specific test file
```

## Linting & Formatting

### Rust
```bash
cargo fmt
cargo clippy --all-features
```

### Web
```bash
cd web
yarn lint                      # ESLint
yarn prettier -w .             # Format with Prettier
```

## Architecture

### File Types
The library handles three SII file formats (identified by 4-byte headers):
- **SCSC** (`ScsC`) - Encrypted and compressed, requires AES-256-CBC decryption then DEFLATE decompression
- **BSII** (`BSII`) - Binary format, requires parsing and conversion
- **SIIN** (`SiiN`) - Text format, already decoded (passthrough)

### Core Modules
- `src/file_type.rs` - File type detection and main decode entry point
- `src/scsc_parse.rs`, `src/scsc_file.rs` - SCSC decryption/decompression
- `src/bsii_parse.rs` - Binary parsing using nom combinators
- `src/bsii_output.rs` - BSII to text output formatting
- `src/wasm.rs` - WebAssembly bindings (behind `wasm` feature flag)

### Web Architecture
- `web/src/App.tsx` - Main React component with file upload UI
- `web/src/decode.worker.ts` - Web Worker for non-blocking decoding
- Tests mock the worker to avoid WASM dependency
