# Contributing to Zap :building_construction: 

First of all, thanks for taking the time to contribute! :heart::tada::+1:

## Getting started

Zap is built with Rust at the moment. There's plans to bootstrap it in
Erlangn in a way that yields a single binary, and that project will be built
with Zap, but this is a little down the road.

For the time being, this means you need the Rust toolchain to work on it. You
can get it here: https://rustup.rs

The `stable` toolchain should be enough.

You can then build and bootstrap the system by running:

```sh
zap $ make build
zap $ make test
zap $ make install
```
