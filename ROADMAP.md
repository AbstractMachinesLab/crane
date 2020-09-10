# Roadmap

This is mostly some ideas that I've been toying around with, in _some_ order.

## Done (or at least prototyped)

* Composable: throw a crane file anywhere and it'll get built.

* Explicit Dependency Graph with Target Resolution

* Rule Library: can build erlang source code!

* Rule Shell: can run a shell target and it'll load the right dependencies.

* Incremental Graph Compilation: only rebuild what has changed

* Split sources and includes for libraries

* Preliminary build of the entire RabbitMQ repositories as a benchmark

* Bootstrap a few languages: Gleam, Clojerl, and Elixir got library rules too

* Tiny polyglot example!

* SPEC: parallel build graph execution: make sure the workers behave

* Toolchain Management: specify the language archive and sha1 and it'll build it and version it for you

* crane clean -- if running this fixes your build, then there's a bug

## Upcoming (in *some* order)

### Actions

- [ ] crane new -- bootstrap an empty project

- [ ] crane test -- run a test target

- [ ] crane lift -- given a rebar3/erlang.mk/mix project, turn it into a crane
  project

- [ ] crane fmt -- format all the sources

### Specification

- [ ] cache: how exactly should the caching work? can we distribute the work?

### Hermeticity

- [ ] Fully Hermetic Builds -- do not rely on system wide installed `wget` or others

- [ ] Build Lock -- only one build at a time allowed to prevent hermeticity breakaga
  without having to move __all the necessary artifacts__ out of source before building.

  This lock would also block fetching stuff, updating dependencies, or anything that
  produces some artifacts.

### Language / Runtime Support

- [ ] More languages?
  - [ ] Luerl both as embedded source
  - [ ] Hamler
  - [ ] LFE

- [ ] Swappable runtime: what if we could make a target that compiles with Lumen
  instead of the standard ERTS?

- [ ] on crane build running typechecks: can we break apart dialyzer into the build
  graph? if the source hasn't changed, the types haven't changed. We can mimic
  OCaml's .cmt[i] files for a stable, cacheable typed representation of modules
  -- could this extend to Gleam/Hamler?

### Dependency Management

- [ ] crane fetch -- download everything that needs to be downloaded, including
  toolchains and dependencies

- [ ] Dependency Flattening

- [ ] Fuzzying

### Rules

- [ ] Shell: custom options and custom shells

- [ ] Test Rules

- [ ] Releases: what constitutes a modern BEAM release

- [ ] Cross-platform releases: since we're just building bytecode, we should be
  able to pull in different runtimes and bundle them for cross-compilation

- [ ] Container rules: `crane build //my_service:container` and `crane run
  //my_service:container`

### Pipe dreams

- [ ] Could we just use the Lumen Erlang/Core->BEAM compiler instead?

### Distribution

- [ ] Cross-compile crane for several architectures
