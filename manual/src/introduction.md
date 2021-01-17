# Introduction

⚡ Zap is a build system built for the modern polyglot teams.

It's built on [Deno 🦕](https://deno.land) and [Rust 🦀](https://rust-lang.org),
and maintained by [Abstract Machines](https://github.com/AbstractMachinesLab).

Check out the [CHANGELOG](./changelog/v0.4.2.md) to see what's new.

## Features

* Ships only a single executable.
* Fast by default. Every step of every build is globally cached and reused.
* Designed to be extended. Custom rules and toolchains can be written quickly
  and easily in Javascript.
* Automatically manages language installations per project for you.
* Easy to get started.

## Philosophy

Zap aims to make working with polyglot monorepos productive and easy.

Monorepos are great for large polyglot codebases, but the tooling to work
efficiently with them is either ecosystem specific (Dune, Lerna), or requires a
large effort upfront (Bazel, Pants, Buck).

Zap aims to be a drop-in solution, so it embraces well-known ecosystem
expectations and conventions to make it as easy as possible to get started.

## Goals

* Only ship a single executable (`zap`)
* Provide a simple, fast, and unified experience for polyglot monorepos:
  * manage language and dependency versions per project
  * build, test, and distribute polyglot projects
* Never hear "it works on my machine" again
