# Building Rust library

Install Rust (e.g. using `rustup`), and run `cargo build`.

If you're developing the Web Assembly interfaces, pass an additional argument of
`--feature wasm` or `--all-features`.

## Running the executable

`cargo run [FILE_TO_DECODE]`

The output will be printed at your standard output.

## Testing and linting

1. Run `cargo test` (optionally `--all-features`) to test the code.
1. Run `cargo fmt` to format code.
1. Run `cargo clippy` to check for linting issues.

# Building Web Assmebly package

1. Install wasm-pack using `cargo install wasm-pack`.
1. Run `wasm-pack build --all-features`.
1. Package will be built at `pkg` directory.

## Testing Web Assembly code

Run `wasm-pack test --chrome --headless --all-features` (replace `--chrome` with other browser of your choice if you wish)

# Building Web Interface

The web interface uses Vite with React.
See [README](web/README.md)

## Testing and linting

1. Install playwright dependencies using `yarn playwright install`
1. Run `yarn test`. By default, `vitest` will run in watch mode.
1. Run `yarn lint` for linting using `eslint`.
1. Run `yarn prettier -w .` to format code. (Currently no automatic formatting enforcement in place.)
