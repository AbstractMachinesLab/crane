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

* Toolchain Management

* Hermetic Builds

* crane fetch -- download everything that needs to be downloaded, including toolchains and dependencies

* crane build should build the toolchains if they aren't ready

* crane clean

* crane new -- bootstrap an empty project

* crane test.unit
* crane test.common
* crane test.proper

* crane lift -- given a rebar3/erlang.mk/mix project, turn it into a crane project

* crane fmt -- format all the sources

* Parallel Build Graph Execution

* Rules:
  * Shell: custom options and custom shells
  * Build: custom erlc/elixirc/gleamc/clojerl/etc options
  * Test

* Releases: what constitutes a modern BEAM release?

* Dependency Flattening and Fuzzying
