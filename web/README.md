# Web Interface

## Setting up dev environment

1. Install `wasm-pack`

```bash
$ cargo install wasm-pack
```

1. Build Web Assembly package (in project root directory)

```bash
$ wasm-pack build --all-features
```

1. The Web Assembly package should be available at `pkg` directory. Enter the `web directory` and install dependencies.

```bash
$ cd web
$ yarn
```

1. The development server can be started via:

```bash
$ yarn dev
```
