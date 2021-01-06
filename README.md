# :zap: zap
> a friendly, fast, and correct build system 

[![Build Status](https://travis-ci.org/AbstractMachinesLab/zap.svg?branch=main)](https://travis-ci.org/AbstractMachinesLab/zap)

## Getting Started

You can download your release from the [releases
page](https://github.com/AbstractMachinesLab/zap/releases). That's it!

Zap will figure out what toolchains it needs (Erlang, Elixir, Caramel, Gleam,
etc) and make sure to install the right ones in a sandbox. This means your
global environment is not modified, and you get to use the exact versions of
the toolchains your project needs.

## Concepts

There's only a handful of concepts to get familiar with that will help us
understand how to use Zap more effectively. These are mirroring the concepts
that the Bazel build tool has already proven to work at scale, but they are
coloured with the flavors of Dune, that provides an instant set up.

The concepts are:

* Workspace
* Package
* Target
* Rule
* Toolchain


1. **Workspace** -- this is the dir in your filesystem with all of your
   project(s) source code. Typically, this will be your entire Git repository.

   Normally you'll have a `Workspace.toml` file at the root of your repo. We
   refer to this as the "workspace root".

2. **Package** -- code is organized in _packages_, which are named collections
   of source files and rules on how to build them. These rules may specify
   dependencies on other packages.

   1 Package corresponds to 1 `zap` file.

   As an example, the following structure defines 2 packages using their local
   paths from the Workspace root.

   1. `//my/app`, and
   2. `//my/app/test`

   ```
   <workspace root>
   └── my
     └── app
       ├── zap
       ├── data
       ├── main.ex
       └── test
         ├── zap
         └── main_test.exs
   ```

   You have full control on how you organize packages so it can best reflect
   the structure of your project(s).

   The more granular packages you have, the faster incremental rebuilds can be.

3. **Target** -- as we saw above, every Package has a number of named
   definitions. These we call _Targets_ and they represent the specific things
   being built or run within a Package.

   A target always consists of:
   * a `rule`,
   * a `name`,
   * optional dependencies (`deps`)

   Depending on the `rule`, the target may have more parameters.

   For example, library targets have tend to have sources (`srcs`), while test
   targets have a single source (`src`). All rules have ecosystem aware defaults.

   Here's an Elixir library and its corresponding test, also written in Elixir.

   ```toml
   # file: <workspace-root>/my/app/zap
   [[elixir_library]]
   name = "lib"
   srcs = [ "*.ex" ]  # this isn't needed, as it is the default

   # file: <workspace-root>/my/app/zap/test
   [[elixir_test]]
   name = "main_test"
   src = "main_test.exs"
   deps = [":lib"]
   ```

   The `name` of a target defines its **Label**. Label's are package paths, as
   we saw above, with the target name at the end.

   In this case, because the package path of the library is
   `<workspace-root>/my/app`, the Label will be `//my/app:lib`.

   We use `//` as a shorthand for `<workspace-root>`.

4. **Rules** -- define how to go from a set of inputs to a set of outputs. The
   name of the rule defines what actions will be taken.

   So for example, an `[[elixir_library]]` rule will be different from an
   `[[gleam_library]]` rule, even if they both create the same set of outputs.

   We're following a pattern of naming that prefixes common rules with the
   language they support:

   * `*_library` for library
   * `*_test` for tests
   * `*_shell` for REPLs (whenever available)

   Which yields:

   * `erlang_library`, `elixir_library`, etc
   * `erlang_test`, `elixir_test`, etc
   * `erlang_shell`, `elixir_shell`, etc

5. **Toolchain** -- many rules share certain tools to perform their work. For
   example, all the Elixir rules need a working Elixir installation.

   It would be too much work to specify that every target depends _also_ on the
   specific language toolchain, so instead we deal with this at the Rule level.

   All rules define what toolchains they rely on, and Zap will build and cache
   them like any other target.

   All toolchains are registered 


