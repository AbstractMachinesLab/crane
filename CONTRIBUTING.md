# Contributing to Crane :building_construction: 

First of all, thanks for taking the time to contribute! :heart::tada::+1:

## Getting started

Crane is built with Rust at the moment. There's plans to bootstrap it in
Erlangn in a way that yields a single binary, and that project will be built
with Crane, but this is a little down the road.

For the time being, this means you need the Rust toolchain to work on it. You
can get it here: https://rustup.rs

The `stable` toolchain should be enough.

You can then build and bootstrap the system by running:

```sh
crane $ make build
crane $ make test
crane $ make install
```

## Codebase

The Caramel codebase is structured to explore a few things:

* `./src/bin` -- the `caramelc` binary

* `./src/erlang` -- a library for working with Erlang and Core Erlang code in
  OCaml including lexers, parsers, ASTs, and invariant checkers

* `./src/compiler` -- includes the compilation pipelines

* `./src/typing` -- includes the type checking pipelines 

* `./src/stdlib` -- the standard libraries shipped with Caramel

