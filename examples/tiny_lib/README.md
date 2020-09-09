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
19:51:53 INFO :: Planning build...
19:51:53 INFO :: Building target: //...
19:51:54 INFO :: Built 4 artifacts in 943ms
```

Subsequent builds after the clean one are heavily and correctly cached:

```
examples/tiny_lib λ crane build
19:52:23 INFO :: Planning build...
19:52:23 INFO :: Building target: //...
19:52:23 INFO :: Built 0 artifacts in 1ms
```

## Running

We can run the `//math:shell` target to get an open REPL where we can use all of
the dependency BEAM files:

```
examples/tiny_lib λ crane run //math:shell
19:53:12 INFO :: Workspace: tiny_lib
19:53:12 INFO :: Target: //math:shell
19:53:12 INFO :: Planning build...
19:53:12 INFO :: Building target and dependencies: //math:shell
19:53:12 INFO :: Built 0 artifacts in 1ms
19:53:12 INFO :: Running target:
Eshell V11.0.3  (abort with ^G)
1> math:test().
16384.0
```
