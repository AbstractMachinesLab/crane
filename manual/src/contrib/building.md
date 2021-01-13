# Building from Source

Zap is built with Rust, so it needs a working installation of the Rust toolchain.
If you are using `rustup`, the `stable` toolchain should suffice.

Additionally, we use `make`, since it makes our CI steps pretty minimal.

### Getting Started

The next scripts show how to set up Zap and create a release:

```sh
# install rust first
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# clone the repository
git clone git@github.com:AbstractMachinesLab/zap.git
cd zap

# install all required dependencies
make setup

# compile projects and runs the tests
make test

# installs project from sources
make install
```

### Common Tasks

To bootstrap the repository, you can run `make setup build`.

To compile the manual, you can run `make manual`.

To run all the tests, you can run `make test`.

To install the local version, you can run `make install`.

To format all the code, you can run `make fmt`.

To create a release of Zap, you can run `make release`, or `make release.win` on
Windows.
