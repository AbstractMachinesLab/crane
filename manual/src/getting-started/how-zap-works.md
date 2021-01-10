# How Zap Works

> NOTE: you do not need to be familiar with Rules and Toolchains to _use_ Zap,
> but understanding them can help you use it more efficiently.

There's only a handful of things to get familiar with that will help us
understand how to use Zap more effectively.

These are mirroring concepts that the Bazel build tool has already proven to
work at scale, but they are coloured with the flavors of Dune, that provides an
instant set up.

They are 5:

* Workspace
* Package
* Target
* Rule
* Toolchain


### Workspace

This is the dir in your filesystem with all of your project(s) source code.
Typically, this will be your entire Git repository.

Normally you'll have a `Workspace.toml` file at the root of your repo. We refer
to this as the "workspace root".

### Package

Code is organized in _packages_, which are named collections
of source files and rules on how to build them. These rules may specify
dependencies on other packages.

1 Package corresponds to 1 `Build.toml` file.

As an example, the following structure defines 2 packages using their local
paths from the Workspace root.

1. `//my/app`, and
2. `//my/app/test`

```
<workspace root>
├── Workspace.toml
└── my
  └── app
    ├── Build.toml
    ├── data
    ├── main.ex
    └── test
      ├── Build.toml
      └── main_test.exs
```

You have full control on how you organize packages so it can best reflect
the structure of your project(s).

The more granular packages you have, the faster incremental rebuilds can be.

### Target

As we saw above, every Package has a number of named definitions. These we call
_Targets_ and they represent the specific things being built or run within a
Package.

A target always consists of 2 things:
* a `rule`, and
* a `name`,

Depending on the `rule`, the target may have more parameters.

For example, library targets have tend to have sources (`srcs`), while test
targets have a single source (`src`). All rules have ecosystem aware defaults.

Here's an Elixir library and its corresponding test, also written in Elixir.

```toml
# file: <workspace-root>/my/app/Build.toml
[[elixir_library]]
name = "lib"
srcs = [ "*.ex" ]  # this isn't needed, as it is the default

# file: <workspace-root>/my/app/test/Build.toml
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

### Rules

Rules define how to go from a set of inputs to a set of outputs. The name of
the rule defines what actions will be taken.

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

### Toolchains 

Many rules share certain tools to perform their work. For example, all the
Elixir rules need a working Elixir installation.

It would be too much work to specify that every target depends _also_ on the
specific language toolchain, so instead we deal with this at the Rule level.

All rules define what toolchains they rely on, and Zap will build and cache
them like any other target.
