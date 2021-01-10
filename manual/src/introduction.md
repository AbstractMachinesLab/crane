# Introduction

⚡ Zap is a build system built for the modern polyglot teams.

It's built on Deno and Rust.

## Features

* Fast by default &mdash; never wait on work that didn't need to be done.
* Built to be Extended &mdash; write new rules and toolchains in Javascript.
* Automatic Toolchains &mdash; manage language installations per project.

## Philosophy

Zap aims to make working with polyglot monorepos productive and easy.

Monorepos are great for large polyglot codebases, but the tooling to work
efficiently with them is either ecosystem specific (Dune, Lerna), or requires a
large effort upfront (Bazel, Pants, Buck).

Zap aims to be a drop-in solution.

## Goals

* Only ship a single executable (`zap`)
* Provide a simple, fast, and unified experience for polyglot monorepos:
  * manage language and dependency versions per project
  * build, test, and distribute polyglot projects
* Never hear "it works on my machine" again
* Extending it with Toolchains and Rules should be easy

## Comparisons

### Zap and rebar3

### Zap and mix
