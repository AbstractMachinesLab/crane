# Roadmap

This is mostly some ideas that I've been toying around with, in _some_ order.

## What's Done (or at least prototyped)

- [X] Composable: throw a crane file anywhere and it'll get built.

- [X] Explicit Dependency Graph with Target Resolution lets you narrow into
  exactly what you want to build (or build the whole thing)

- [X] Rule Library: can build erlang source code!

- [X] Rule Shell: can run a shell target and it'll load the right dependencies.

- [X] Incremental Graph Compilation: only rebuild what has changed

- [X] Split sources and includes for libraries

- [X] Preliminary build of the entire RabbitMQ repositories as a benchmark

- [X] Bootstrap a few languages: Gleam, Clojerl, and Elixir got library rules
  too

- [X] Tiny polyglot example! We're building some gleam, some clojerl, some
  elixir, and some erlang ties it all together. Try it out in the shell.

- [X] SPEC: parallel build graph execution: make sure the workers behave

- [X] Toolchain Management: specify the language archive and sha1 and it'll
  build it and version it for you

- [X] crane clean -- if running this fixes your build, please file an issue
  (that should be a bug)

- [X] Release cross-compiled binaries for common architectures

## Upcoming (in *some* order)

### Goals (things Crane can do)

- [ ] crane query -- find targets in the build graph

- [ ] crane build -- fetch, lift, and build dependencies too

- [ ] crane new -- bootstrap an empty project.

- [ ] crane test -- run a test target

- [ ] crane lift -- given a rebar3/erlang.mk/mix project, turn it into a crane
  project

- [ ] crane fmt -- format all the sources

### Build Graph Execution

- [ ] Parallel build graph execution!

- [ ] Can we distribute it?

### Specification

- [ ] cache: how exactly should the caching work? can we distribute the work?

- [ ] distributed build

### Hermeticity

- [ ] Fully Hermetic Builds -- do not rely on system wide installed `wget` or others

- [ ] Build Lock -- only one build at a time allowed to prevent hermeticity breakaga
  without having to move __all the necessary artifacts__ out of source before building.

  This lock would also block fetching stuff, updating dependencies, or anything that
  produces some artifacts.

- [ ] sandboxing -- we are currently hoping the tools will not generate other
  outputs on the directory they are being invoked from, but this can't always
  be guaranteed! We should instead prepare a sandbox where we can detect unwanted
  outputs (e.g, sandbox is temp folder, its empty, we run command, only expected
  files are there) and we can complain loudly about them.

  As a side-effect, this allows us to easily just tarball up the entire output
  and move it around more easily. This should simplify the transitive input
  gathering too.

  As a litmus test, if we need to add anything new to a .gitignore because a tool
  had some side-outputs, that's an output we need to declare!

  As an actual enforcing rule: cgroups? look into how to run the command as a user
  that can only read and write to the sandbox.

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
  toolchains and dependencies, but does no building.

- [ ] Dependency Flattening

- [ ] Fuzzying

### Rules

- [ ] Shell: custom options and custom shells

- [ ] Test Rules

- [ ] Releases: what constitutes a modern BEAM release

- [ ] Cross-platform releases: since we're just building bytecode, we should be
  able to pull in different runtimes and bundle them for cross-compilation

- [ ] Container rules: `crane build //my_service:container` and `crane run //my_service:container`

### Pipe dreams

- [ ] Could we just use the Lumen Erlang/Core->BEAM compiler instead?

- [ ] Rewriting in Erlang and compiling to a binary may be possible with either
  Lumen or Caramel -- parallelism with Caramel would be trickier until OCaml
  multicore!

### Distribution

...
