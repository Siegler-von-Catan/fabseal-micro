# FabSeal Server

## Requirements

- A working Rust toolchain.
  See https://www.rust-lang.org/tools/install for instructions on how to install Rust.
- Redis server

## Installation

1. Use `cargo build` for a debug build (or `cargo build --release` for a release build)
2. After the build, the `target/debug` folder will contain the following binaries:
    - `fabseal-micro`: This is the main HTTP server
    - `fabseal-worker-blender`: This is the Blender worker

## Running the server

Steps:

- Start the Redis server if it is not already running
- Start `fabseal-micro` (e.g. `cargo run --bin fabseal-micro`)
- Start `fabseal-worker-blender` (e.g. `cargo run --bin fabseal-worker-blender`)

## Configuration

`fabseal-micro` and `fabseal-worker` can be configured using settings in
a file with the name `config.toml`.
An example configuration file is included in `config/default.toml`:
```toml
debug = true
dmstl_directory = "path/to/displacementMapToStl"

[http]
endpoint = "127.0.0.1:8080"
cookie_domain = "localhost"

[redis]
address = "127.0.0.1:6379"
# db_id = 0
# password = "foobar"

[limits]
queue_limit = 32
image_ttl = 600
result_ttl = 600
session_ttl = 1200
```

* `debug`: Set to `true` for local development (disables some Cookie security options)
* `dmstl_directory`: Path to the directory containing [displacementMapToStl](https://github.com/Siegler-von-Catan/displacementMapToStl)
* `http.endpoint`: Set the HTTP endpoint for `fabseal-micro`
* `http.cookie_domain`: Cookie domain name for `fabseal-micro`
* `redis.address`: Address of Redis server (default should work for local development)
* `redis.password`: Password for Redis authentication (currently unimplemented!)
* `limits.queue_limit`: Work queue task limit
* `limits.image_ttl`: TTL in seconds for (input) images
* `limits.result_ttl`: TTL in seconds for result files (STL)
* `limits.session_ttl`: TTL in seconds for sessions


## Example

The script `demo.sh` contains an example of a typical invocation of the Create API (using [curl](https://curl.se/)).
A typical curl output is included in `demo_output.txt`.