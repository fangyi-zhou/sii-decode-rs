# A Rust library to decode SII files

This library provides functionalities to decode SII files ("Unit serialized
file") that are used in SCS Software games (e.g. Euro Truck Simulator 2).

This library is ported from existing SII decrypt software by [František
Milt (SII_Decrypt)](https://github.com/TheLazyTomcat/SII_Decrypt) and [Joshua
Menzel (sii-decryptsharp)](https://gitlab.com/jammerxd/sii-decryptsharp).
Many thanks for their investigation and implementation work.

This library is also available in an executable format that takes a file of an
supported format and outputs the decrypted/decoded result.

## Usage

You can use the web interface at https://sii-decode.github.io/

## Technical Notes

The `src` directory contains the source code of the Rust library. The library
handles 3 types of files, identified by their header types: scsc (encrypted,
compressed data files), bsii (binary data files), and siin (textual data types).

The Rust library is then compiled into Web Assembly using `wasm-pack`, so that
the library can be used in browsers. The `web` directory contains the source
code of the web interface. This allows the decoding work to be performed in the
browser, so that users can use the tool in their browser without the need to
upload their files to a server.

## Contributing

See [HACKING.md](./HACKING.md).

## Roadmap

- [ ] Implement a proper CLI for the binary tool.
- [ ] Add unit tests for serialising BSII files
- [ ] Define the WebAssembly API
- [X] Provide the tool as a web application (using WebAssembly)
- [ ] Use the decode result to make an achievement tracker tool
