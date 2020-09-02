# Roadmap

### Done (or at least prototyped)

* Composable: throw a crane file anywhere and it'll get built.

* Explicit Dependency Graph with Target Resolution

* Shell: can run a shell target and it'll load the right dependencies.

* Incremental Graph Compilation: only rebuild what has changed

* Split sources and includes for libraries

* Preliminary build of the entire RabbitMQ repositories as a benchmark

* Bootstrap a few languages: Gleam, Clojerl, and Elixir

* Tiny polyglot example!

### Upcoming (in no particular order)

* SPEC for the cache

* Toolchain Management

* Hermetic Builds

* More languages:
  * Luerl both as embedded source 
  * Hamler
  * LFE

* crane fetch -- download everything that needs to be downloaded, including
  toolchains and dependencies

* crane build should:
  * build the toolchains if they aren't ready
  * build all dependencies that are needed
  * run typechecks

* on crane build running typechecks: can we break apart dialyzer into the build
  graph? if the source hasn't changed, the types haven't changed. We can mimic
  OCaml's .cmt[i] files for a stable, cacheable typed representation of modules
  -- could this extend to Gleam/Hamler?

* crane clean -- if running this fixes your build, then there's a bug

* crane new -- bootstrap an empty project

* crane test.unit
* crane test.common
* crane test.proper

* crane lift -- given a rebar3/erlang.mk/mix project, turn it into a crane
  project

* crane fmt -- format all the sources

* Parallel Build Graph Execution

* Rules:
  * Shell: custom options and custom shells
  * Build: custom erlc/elixirc/gleamc/clojerl/etc options
  * Test

* Releases: what constitutes a modern BEAM release? Can we really achieve super
  minimal releases by fuzzying even OTP out of the way?

* Dependency Flattening and Fuzzying

* Container rules: `crane build //my_service:container` and `crane run
  //my_service:container`

* Cross-platform releases: since we're just building bytecode, we should be
  able to pull in different runtimes and bundle them for cross-compilation?

* Swappable runtime: what if we could make a target that compiles with Lumen
  instead of the standard ERTS?
