# Lattice

A stack-based programming language with a [very large](# "2^32 by 2^32 cells") grid of cells as its memory structure written
in Rust. **Heavily** inspired by [Porth](https://gitlab.com/tsoding/porth). This is not meant to be a 
usable programming language (yet), so features are subject to change. Lattice is mainly a personal
experiment in how compilers work, and how easy (or difficult) it is to write one in Rust.

## Goals

- [x] Compiled and Interpreted
- [ ] "Infinite" 2d Grid of Byte-Sized Cells
- [x] [Turing Completeness](https://en.wikipedia.org/wiki/Turing_completeness)
    - See the [Rule 110 Example](./examples/rule-110.lat)
- [ ] [Self-hosting](https://en.wikipedia.org/wiki/Self-hosting_(compilers))
- [ ] Cross-platform (Compiled)

## Getting Started

This language is built with Rust, so it relies on `cargo` for building.
```console
$ git clone https://github.com/CoBrooks/lattice && cd lattice
$ cargo build --release
$ ./target/release/lattice <sim | com> [FILE.lat]
```

Note: Lattice currently only compiles to a x86_64 ELF binary, 
and thus can only be run on Linux (for now).

See the [tests](./tests/) and [examples](./examples/) for example syntax and logic.

## TODO:

- [x] Conditions and Loops
- [x] Implement memory structure
    - [x] 2d grid
    - [x] Traversal of the grid (steps and jumps)
    - [x] Storing values within the grid
    - [x] Some form of pointers
- [ ] Directional arrays (C-style arrays; pointer to the start + a direction, ends with a null byte)

