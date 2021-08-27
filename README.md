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
- Start `fabseal-worker-blender`

## Configuration

`fabseal-micro` and `fabseal-worker`
can be configured using settings in `config/default.toml`.
An example configuration file is included:
```toml
debug = true
http_endpoint = "127.0.0.1:8080"
# domain = "localhost"
dmstl_directory = "path/to/displacementMapToStl"

[redis]
address = "127.0.0.1:6379"
```

* `debug`: Set to `true` for local development (disables some Cookie security options)
* `http_endpoint`: Set the HTTP endpoint for `fabseal-micro`
* `domain`: Cookie domain name for `fabseal-micro`
* `dmstl_directory`: Path to the directory containing [displacementMapToStl](https://github.com/Siegler-von-Catan/displacementMapToStl)
* `redis.address`: Address of Redis server (default should work for local development)

## Example

The script `demo.sh` contains an example of a typical invocation of the Create API (using [curl](https://curl.se/)).
A typical curl output is included in `demo_output.txt`.