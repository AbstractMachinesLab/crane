# :building_construction: crane
> a build system for the BEAM, focused on speed, correctness, and developer
> productivity.

<img src="https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Fi.pinimg.com%2Foriginals%2Fb9%2F3d%2Fb5%2Fb93db5e965fb69dddf7e672ed5f74395.jpg&f=1&nofb=1" />

Crane is a build system for BEAM projects. You can use it to build executables,
libraries, run tests, run local consoles, create releases, build container
images for releases on different platforms, and much more.

## Why another build system?

First of all, Rebar3, Erlang.mk, and Mix are all fantastic tools with tons of
great engineering and TLC put into them by their communities. I sincerely thank
you all for them.

I'm building Crane because I've got a few itches that these tools have not
helped me scratch, and I've decided that I'd start anew rather than nudge any
of them in some direction, because I'd like to rethink a few things from zero.

For starters, building large projects with more than a single repository is
painful. When it gets to dozens or hundreds of repositories of code it is
unbearable. There's a lot of meta-build-system work to do: rely on git
submodules, drop version constraints dynamically, flatten them at the top level
anyway, rename stuff, you name it. Not to mention the indirection layers that I
have to navigate as a newcomer to the project, first to find code and second to
build code. _Why can't I just work on the entire thing at once, but just build
the tiny bits that I'm working with?_

Now once you've managed to build the thing, you decide to test it. And some say
you should test thing in isolation, but some still want to run all the tests
when stuff is together, "just in case". _Why can't I just run all the tests
once, and then only retest the things that actually changed?_

Normally we reach out of these build tools to, after creating a release,
package it in a container image. And you gotta figure out what image is the
right one, with the right ERTS version, and if you're starting from something
small like Alpine then you gotta make sure you get the ERTS .so dependencies
(like ncurses?), and whatnot. _Why can't the build system, who already knows
everything that is needed to know about my project, build the container image?_

And a few more like this that I'll talk more about below.

I think we've gotten used to these things and we're quite productive
nonetheless, but I'd like to take a step back and rethink how we could do them
differently.

After using dune, bazel, and bsb, for a couple of years, I know these things
can be done.

So I intend to do them.

## Ok, so how exactly is this different than Rebar3/Mix/Erlang.mk/etc?

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

For example, Crane is built with **support for humongous repositories of code**.
This is baked into the nature of the build system, rather than a post-hoc
add-on, and it shows.

Crane doesn't lean towards splitting projects into several repositories with
their own entire build config and inter-repo versioning. Instead, you are
encouraged to have a single repo, define a workspace only once and just mark
what directories are libraries/binaries/releases of what kind, and what
dependencies they have on each other.

With this explicit build graph, we can ask Crane to build specific things, and
it'll know just exactly what graph paths need building. Crane is lazy, and so
it **only does the minimum amount of work needed to satisfy your build goal**.

What's more, Crane is so lazy, it **uses a content-addressable cache** of every
node in the build graph so if the inputs haven't changed, the output is just a
cache hit. While the cache is now local, I'm working to make it distributed for
large teams to share build artifacts.

This means that a single step that takes 30 minutes to build, won't need to be
redone unless some of its inputs have absolutely changed. And yes this includes
tests and releases too. Every build target.

#### Ecosystem Aware

The BEAM is a platform for several languages, and we've done an adequate job at
supporting polyglot projects. Crane **supports polyglot BEAM projects with
seamless interop**, to the extent that they may make some names wonky for each
other (looking at you Elixir), by already integrating the language toolchains
or making it very easy to integrate new ones.

If it can output valid BEAM files, chances are it'll just work and you'll be
able to include it as a dependency on some of your libraries, tests, shells, or
releases.

Dependency resolution and versioning are also issues that when we're working
with smaller repositories are just not as obvious. Crane brings this up and
front by making the **entire workspace shares a dependency tree**. This
ensures we're not accidentally running multiple conflicting versions of
anything, and allows us to do some cool things like fuzzying out dependencies
to keep our workspaces clean.

Typically we reach outside of our usual BEAM build tools to pack our released
code, but, whether you're running kubernetes or docker-compose, you're likely
running your code in a container. Crane aims to **support distroless
containerized releases** without even depending on Docker.

#### Hermetic

Crane aims to support **fully reproducible builds**, including deterministic
container images, to **support differential deployments**. So if the source
hasn't changed, the image hash shouldn't either. Whatever CI deploys should
have the exact same hash that you came up with in your machine.

This means that Crane won't let system-wide pre-installed tools do any work,
and it will manage and version any language toolchains necessaries to do these
builds. If it doesn't right now, it is just a matter of time until it does.

## Getting Started

...

## Installing

Right now you can only install Crane from source, which means you need a Rust
toolchain. Running `cargo install --path .` should do the trick, and it should
work on the latest stable toolchain.
