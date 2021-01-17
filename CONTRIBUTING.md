# Contributing to :zap: Zap

First of all, thanks for taking the time to contribute! :heart::tada::+1:

## Getting started

Zap is built with Rust, so you need the Rust toolchain to work on it. You can
get it here: https://rustup.rs

The `stable` toolchain should be enough.

You can then build and bootstrap the system by running:

```sh
zap $ make build
zap $ make test
zap $ make install
```

## Release Checklist

1. Make sure CI is happy with the current commit build here:

https://github.com/AbstractMachinesLab/zap/actions?query=branch%3Amain

2. Make sure all the version numbers have been bumped

```
sed -i '/version/s/${OLD_VERSION}/${NEW_VERSION}/g' $(rg "\[package\]" -l)
```

3. Write a changelog entry in the manual at `./manual/src/changelog/${NEW_VERSION}`

4. Update SUMMARY.md and Introduction.md manual pages to point to the new changelog entry.

5. TAG the build. Push the tag, and wait for CI.

6. Edit the Draft Release, copy over the changelog entry in the release body.

   MAKE SURE TO ADD `<div id="assets"></div>` AT THE BOTTOM OF THE RELEASE BODY

   Save as regular Release (not pre-release).

7. Run `cargo publish` on the packages in the following order:

   1. zap-buildscript
   1. zap-core
   1. zap-build-engine
   1. zap-bin

8. Go tell your friends!
