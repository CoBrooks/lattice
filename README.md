# Lattice

A stack-based programming language with an "infinite" grid of cells as its memory structure written
in Rust. **Heavily** inspired by [Porth](https://gitlab.com/tsoding/porth). This is not meant to be a 
usable programming language (yet), so features are subject to change. Lattice is mainly a personal
experiment in how compilers work, and how easy (or difficult) it is to write one in Rust.

## Goals

- [x] Compiled and Interpreted
- [ ] "Infinite" 2d Grid of Byte-Sized Cells
- [ ] [Turing Completeness](https://en.wikipedia.org/wiki/Turing_completeness)
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

### Operators

- print: prints an unsigned integer
```
4 print
```

- add, subtract, multiply, and divide
```
2  2 + print
8  4 - print
2  2 * print
16 4 / print
```

- dup: duplicates the top element of the stack
```
2 dup + print
```

## TODO:

- [ ] Implement memory structure
    - [ ] 2d grid
    - [ ] Traversal of the grid (steps and jumps)
    - [ ] Storing values within the grid
    - [ ] Some form of pointers
- [ ] Conditions and Loops
- [ ] Directional arrays (C-style arrays; pointer to the start + a direction, ends with a null byte)

