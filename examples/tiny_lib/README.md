# `tiny_lib`
> A small polyglot BEAM library

This little example showcases how to write a polyglot BEAM library, using: 

* Gleam
* Elixir
* Clojerl
* Erlang

## Building

We can build all of the targets by calling `crane build`:

```
examples/tiny_lib λ crane build
23:27:09 INFO :: Planning build...
23:27:09 INFO :: Readying toolchains: [Erlang, Gleam, Clojure, Elixir]
23:27:09 INFO :: Building target: //...
23:27:10 INFO :: Built 4 artifacts in 886ms
```

Subsequent builds after the clean one are heavily and correctly cached:

```
examples/tiny_lib λ crane build
23:27:29 INFO :: Planning build...
23:27:29 INFO :: Readying toolchains: [Erlang, Elixir, Clojure, Gleam]
23:27:29 INFO :: Building target: //...
23:27:29 INFO :: Built 0 artifacts in 1ms
```

## Running

We can run the `//math:shell` target to get an open REPL where we can use all of
the dependency BEAM files:

```
examples/tiny_lib λ crane run //math:shell
23:27:43 INFO :: Workspace: tiny_lib
23:27:43 INFO :: Target: //math:shell
23:27:43 INFO :: Planning build...
23:27:43 INFO :: Readying toolchains: [Elixir, Clojure, Gleam, Erlang]
23:27:43 INFO :: Building target and dependencies: //math:shell
23:27:43 INFO :: Built 0 artifacts in 1ms
23:27:43 INFO :: Running target:
Eshell V11.0.3  (abort with ^G)
math:test().
1> 16384.0
```

There's a littl wonkyness with the output, but otherwise things are running
fine.
