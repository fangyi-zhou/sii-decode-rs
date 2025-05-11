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

1. Register the built package for development

```bash
$ cd pkg
$ yarn link
$ cd ..
```

1. The Web Assembly package should be available at `pkg` directory. Enter the
`web directory` and install dependencies. Also, link the built Web Assembly
package so that any changes can be picked up.

```bash
$ cd web
$ yarn link sii-decode-rs
$ yarn
```

1. The development server can be started via:

```bash
$ yarn dev
```
