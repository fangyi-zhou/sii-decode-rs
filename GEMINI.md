# GEMINI.md - Project Context

This file provides foundational mandates and context for Gemini CLI when working within this repository.

## Project Overview

`sii-decode-rs` is a Rust-based library and tool for decoding SII files ("Unit serialized files") used in SCS Software games (e.g., Euro Truck Simulator 2, American Truck Simulator). It handles several file formats identified by headers:
- **SCSC** (`ScsC`): Encrypted and compressed data (AES-256-CBC + DEFLATE).
- **BSII** (`BSII`): Binary data format.
- **SIIN** (`SiiN`): Textual data format.

Beyond simple decoding, the library also provides high-level abstractions for extracting and analyzing **game save data**, such as delivery logs and achievement progress.

The project consists of:
- A core **Rust library** (`src/`).
- A **CLI tool** for local file decoding.
- **WebAssembly bindings** for browser-side decoding.
- A **React-based web interface** (`web/`) built with Vite.

## Building and Running

### Rust Library & CLI
- **Build**: `cargo build --all-features`
- **Run CLI**: `cargo run -- [FILE_TO_DECODE]`
- **Test**: `cargo test --all-features`
- **Lint**: `cargo clippy --all-features`
- **Format**: `cargo fmt`
- **Example**: `cargo run --example delivery_log [SAVE_FILE_PATH]`

### WebAssembly (WASM)
- **Install Tool**: `cargo install wasm-pack`
- **Build**: `wasm-pack build --all-features` (outputs to `./pkg`)
- **Test**: `wasm-pack test --chrome --headless --all-features`

### Web Interface
All commands should be run from the `web/` directory:
- **Install Deps**: `yarn`
- **Dev Server**: `yarn dev`
- **Production Build**: `yarn build`
- **Test**: `yarn test` (uses Vitest)
- **Lint**: `yarn lint` (uses ESLint)
- **Format**: `yarn prettier -w .`

## Architecture and Core Modules

- `src/lib.rs`: Library entry point and WASM module declaration.
- `src/main.rs`: CLI binary entry point.
- `src/file_type.rs`: Logic for detecting file types (SCSC, BSII, SIIN) and routing to appropriate decoders.
- `src/scsc_parse.rs` / `src/scsc_file.rs`: Handling of SCSC encrypted/compressed files.
- `src/bsii_parse.rs`: Binary parsing using `nom` combinators.
- `src/bsii_output.rs`: Logic for converting BSII binary data into human-readable text.
- `src/save_data.rs`: High-level typed data structures for game saves (delivery logs, job types, truck brands, and achievement tracking).
- `src/wasm.rs`: WebAssembly interface and worker-friendly API.
- `web/src/decode.worker.ts`: Web Worker that wraps the WASM module for non-blocking UI.

## Development Conventions

- **Rust**: Follow standard Rust idioms. Use `cargo clippy` and `cargo fmt` regularly.
- **WASM**: Ensure all features are tested with `wasm-pack` if making changes to the library.
- **Web**: Use TypeScript for all web development. Follow the existing React component structure.
- **Testing**:
  - Add Rust unit tests in the respective modules.
  - Integration tests in `tests/`.
  - Web tests in `web/tests/` using Vitest and React Testing Library.
- **Documentation**: Update `README.md` or `HACKING.md` if changing the project's structure or build process.
