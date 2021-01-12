# Introduction

⚡ Zap is a build system built for the modern polyglot teams.

It's built on Deno and Rust.

Check out the [CHANGELOG](./changelog/v0.4.0.md) to see what's new.

## Features

* Fast by default. Every step of the build is cached and reused.
* Ships only a single executable.
* Designed to be extended. New rules and toolchains can be written quickly and
  easily in Javascript.
* Automatically manages language installations per project for you.

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
