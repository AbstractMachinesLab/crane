# Armstrong

A new Erlang toolchain.

Core goals:
- Monorepo oriented
- Content-addressable
- Folder structure is automatic namespacing
- Explicit dependency graph
- Incremental compilation
- Configuration as code

Additional goals:
- Build tool interop

## Monorepo oriented

One of the biggest pet peeves I've had with the current Erlang and
Elixir tooling is that of monorepository support.

Sure, umbrella releases have served similar purposes but they are
not quite the same. They are intended to be deployed __together__
in a single Erlang distribution (regardless of the number of nodes
it comprises).

What I'd like to be able to do is to write code for several independent shared
libraries, and several independently deployable applications, all within a
single repository. 

And I'd like this to be handled by a single tool, with a single project
configuration, rather than a number of projects artificially stitched with
relative paths and other UNIX hackery.

It should allow me to estabish clear dependencies between paths of source code,
regardless of the language, to establish hierarchies like:

* A UI test depends on
* A UI module, that depends on
* An HTTP API definition, that depends on
  * A module for a model, that depends on
    * an abstract definition of information
  * The database module, that depends on
    * an external dependency to talk to Postgres

So that when the abstract definition of information changes, we know what code
will be affected.

## Content Addressable

## Folder structure is automatic namespacing

## Incremental compilation

## Configuration as code
Configuration should be treated as another module.

## Language interop
