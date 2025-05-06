# A Rust library to decode SII save files

This library provides functionalities to decode SII files ("Unit serialized
file") that are used in SCS Software games (e.g. Euro Truck Simulator 2).

This library is ported from existing SII decrypt software by František
Milt https://github.com/TheLazyTomcat/SII_Decrypt and Joshua Menzel
https://gitlab.com/jammerxd/sii-decryptsharp.
Many thanks for the initial investigation and the implementation work.

This library is also available in a binary executable tool that takes a file and
outputs the decrypted/decoded result.

## Usage

TODO

## Roadmap

- [ ] Implement a proper CLI for the binary tool.
- [ ] Add unit tests for serialising BSII files
- [ ] Define the WebAssembly API
- [ ] Provide the tool as a web application (using WebAssembly)
