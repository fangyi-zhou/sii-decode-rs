# AGENTS.md

Repository guidance for coding agents working on `sii-decode-rs`.

## Project Overview

This repository contains a Rust library and CLI for decoding SII files used by
SCS Software games, plus a Vite/React web UI that runs the Rust decoder through
WebAssembly. The public web app is published at <https://sii-decode.github.io/>.

The decoder supports three 4-byte file headers:

- `ScsC`: encrypted and compressed payload; decrypt with AES-256-CBC, then
  zlib/DEFLATE decompress.
- `BSII`: binary SII; parse and render to textual SII.
- `SiiN`: textual SII; pass through unchanged.

## Repository Layout

- `src/lib.rs`: Rust library module exports.
- `src/main.rs`: small CLI wrapper around `file_type::decode_until_siin`.
- `src/file_type.rs`: file type detection and the top-level decode flow.
- `src/scsc_parse.rs`, `src/scsc_file.rs`: `ScsC` parsing, decryption, and
  decompression.
- `src/bsii_parse.rs`, `src/bsii_file.rs`, `src/bsii_output.rs`: `BSII` model,
  parser, and textual output formatter.
- `src/wasm.rs`: `wasm-bindgen` API behind the `wasm` feature.
- `tests/wasm.rs`: browser wasm tests, compiled only with `--features wasm`.
- `web/`: React 19/Vite UI. `web/src/decode.worker.ts` calls the wasm package
  from a worker so large decodes do not block the UI.

## Rust Commands

```bash
cargo build
cargo build --all-features
cargo test
cargo test --all-features
cargo test test_name -- --exact --nocapture
cargo fmt
cargo clippy --all-features
cargo run -- path/to/file.sii
```

Use `--all-features` when touching `src/wasm.rs`, `tests/wasm.rs`, crate
exports, or any API used by the web package.

## WebAssembly Commands

```bash
cargo install wasm-pack
wasm-pack build --all-features
wasm-pack test --chrome --headless --all-features
```

`wasm-pack build --all-features` writes the local npm package to `pkg/`. The web
package depends on it with `"sii-decode-rs": "file:../pkg"`, so rebuild `pkg/`
after Rust decoder or wasm API changes before running the real web app build.

## Web Commands

Run these from `web/`:

```bash
yarn
yarn dev
yarn build
yarn test
yarn test App.test
yarn lint
yarn prettier -w .
```

The Vitest setup uses Happy DOM plus Playwright browser mode. The app tests mock
`decode.worker?worker`, so they validate UI behavior without requiring a fresh
wasm package. `yarn build` and manual `yarn dev` runs do require `../pkg` to be
present and current.

## Implementation Notes

- Prefer keeping the Rust decode pipeline centered on
  `file_type::decode_until_siin`; both the CLI and wasm wrapper use it.
- The binary parsers use `nom` and little-endian numeric parsers. Follow the
  existing parser style when adding BSII fields or versions.
- `BSII` structures borrow from the input buffer. Preserve lifetimes and avoid
  unnecessary allocation except where textual output or encoded IDs require
  owned `String`s.
- The CLI is intentionally minimal and currently uses `unwrap()` for failures;
  do not broaden CLI behavior unless the task is about CLI polish.
- The web UI should keep decode work inside `decode.worker.ts`; avoid moving
  wasm decoding onto the React render thread.
- Large decoded files are preview-limited in `App.tsx` while the full content is
  exposed through a Blob download URL. Preserve that behavior when changing the
  UI flow.

## Verification Guidance

- For parser or output changes, run at least `cargo test`; add
  `cargo test --all-features` if wasm-facing code might be affected.
- For wasm API changes, run `wasm-pack test --chrome --headless --all-features`
  when a browser is available.
- For web UI changes, run `yarn test` from `web/`; run `yarn build` after
  rebuilding `../pkg` when the wasm package or Vite integration is involved.
- Keep generated build artifacts such as `target/`, `pkg/`, and web build
  outputs out of commits unless the user explicitly asks for them.
