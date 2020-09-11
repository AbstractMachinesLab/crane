# :building_construction: crane
> a build system for the BEAM, focused on speed, correctness, and developer
> productivity.

<img src="https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Fi.pinimg.com%2Foriginals%2Fb9%2F3d%2Fb5%2Fb93db5e965fb69dddf7e672ed5f74395.jpg&f=1&nofb=1" />

Crane is a build system for BEAM projects. You can use it to build executables,
libraries, run tests, run local consoles, create releases, build container
images for releases on different platforms, and much more.

## How is this different than Rebar3/Mix/Erlang.mk/etc?

Crane is built in the image of [dune](https://dune.build) and
[bazel](https://bazel.build), and some of the buzzwords that those carry have
implications across the board:

* scalable
* fast
* hermetic
* incremental
* composable
* ecosystem-aware

#### Scalable, Incremental, and Fast

For example, Crane is built with *support for humongous repositories of code*.
This is baked into the nature of the build system, rather than a post-hoc
add-on, and it shows.

Crane doesn't lean towards splitting projects into several repositories with
their own entire build config and inter-repo versioning. Instead, you are
encouraged to have a single repo, define a workspace only once and just mark
what directories are libraries of what kind, and what dependencies they have on
each other.

With this explicit build graph, we can ask Crane to build specific nodes, and
it'll know just exactly what graph paths need building. Crane is lazy, and so
it *only does the minimum amount of work needed to satisfy your build goal*.

What's more, Crane is so lazy, it *uses a content-hashed cache* of every node
in the build graph so if the inputs haven't changed, the output is just a cache
hit.

This means that a step that takes 30 minutes to build, won't need to be redone
unless some of its inputs have absolutely changed. And yes this includes tests
and releases too.

#### Ecosystem Aware

The BEAM is a platform for several languages, but no need to take sides. Crane
**supports polyglot BEAM projects to interop seamlessly**, by already
integrating the language toolchains or making it very easy to integrate the. If
it can output valid BEAM files, chances are it'll just work and you'll be able
to include it as a dependency on some of your libraries, tests, shells, or
releases.

Dependencies and versioning are also issues that when we're working with
smaller repositories are just not as obvious. Crane brings this up and front by
making the **entire workspace share a flat dependency tree**. This ensures
we're not accidentally running multiple conflicting versions of anything, and
allows us to do some cool things like fuzzying out dependencies to keep our
workspaces clean.

Typically we reach outside of our usual BEAM build tools to pack our released
code, but, whether you're running kubernetes or docker-compose, you're likely
running your code in a container. Crane aims to **support distroless
containerized releases**.

#### Hermetic

Crane aims to support *fully reproducible builds*, including deterministic
container images, to *support differential deployments*. So if the source
hasn't changed, the image hash shouldn't either!

This means that Crane won't let system-wide pre-installed tools do any work,
and it will manage and version any language toolchains necessaries to do these
builds. If it doesn't right now, it is just a matter of time until it does.

## Getting Started

## Installing

Right now you can only install Crane from source, which means you need a Rust
toolchain. Running `cargo install --path .` should do the trick, and it should
work on the latest stable toolchain.
