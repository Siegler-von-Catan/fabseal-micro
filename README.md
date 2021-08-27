# FabSeal Server

## Requirements

- A working Rust toolchain.
  See https://www.rust-lang.org/tools/install for instructions on how to install Rust.
- Redis server

## Installation

1. Use `cargo build` for a debug build (or `cargo build --release` for a release build)
2. After the build, the `target/debug` folder will contain the following binaries:
    - `fabseal-micro`: This is the main HTTP server
    - `fabseal-worker-blender`: This is the Blender HTTP worker

## Running the server

Steps:

- Start `fabseal-micro`
- Start `fabseal-worker-blender`, setting DMSTL_DIR to the directory containing [displacementMapToStl](https://github.com/Siegler-von-Catan/displacementMapToStl)

## Configuration

`fabseal-micro` can be configured using settings in `config/default.toml`.
