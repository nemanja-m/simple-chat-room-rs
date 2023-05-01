# Simple Chat Server in Rust

Simple multi-threaded HTTP chat server with in-memory state.

## Getting Started

Build chat server with

```bash
cargo build --release
```

Run server with

```bash
./target/release/chat
```

By default, HTTP server starts with 4 threads and listens for new connections at `0.0.0.0:7878`.

### Configuration

```bash
$ ./target/release/chat -h
Usage: chat [OPTIONS]

Options:
      --host <HOST>        [default: 0.0.0.0]
      --port <PORT>        [default: 7878]
      --threads <THREADS>  [default: 4]
  -h, --help               Print help
```

## Docker

Build docker image with

```bash
docker build -t rust-chat-server .
```

and run container with

```bash
docker run --name rust-chat-server --rm -p 7878:7878 rust-chat-server:latest
```
