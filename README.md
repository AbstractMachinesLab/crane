# :building_construction: crane
> a build system for the BEAM, focused on speed, correctness, and developer
> productivity.

[![Build Status](https://travis-ci.org/AbstractMachinesLab/crane.svg?branch=main)](https://travis-ci.org/AbstractMachinesLab/crane)

<img src="https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Fi.pinimg.com%2Foriginals%2Fb9%2F3d%2Fb5%2Fb93db5e965fb69dddf7e672ed5f74395.jpg&f=1&nofb=1" />

Crane is a build system built for the BEAM.

## Getting Started

You can download your release from the [releases
page](https://github.com/AbstractMachinesLab/crane/releases). That's it!

Crane will figure out what toolchains it needs (Erlang, Elixir, Caramel, Gleam,
etc) and make sure to install the right ones in a sandbox. This means your
global environment is not modified, and you get to use the exact versions of
the toolchains your project needs.
