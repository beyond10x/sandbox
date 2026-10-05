---
title: Run a project's tests in the sandbox
sidebar_position: 1
description: Run cargo test inside b10x-sandbox with the Rust toolchain read-only and no network.
lede: A toolchain enters read-only with --ro, and the environment it needs is passed on the command line, because the sandbox starts from a cleared environment with HOME=/tmp.
source: src/confinement.rs (environment baseline), src/cli.rs; run with b10x-sandbox 0.1.0, bubblewrap 0.12.0 and rustup on the host
---

# Run a project's tests in the sandbox

The sandbox starts from a cleared environment: `PATH=/usr/bin:/bin`, `HOME=/tmp`, and nothing
else of yours. A toolchain installed under your home directory is therefore neither mounted nor
found until you say so.

## Rust with rustup

From a crate with no dependencies to download (the sandbox has no network):

```console
$ cd ~/.cache/sandbox-demo/app
$ cargo init --name demo --vcs none .
$ b10x-sandbox --ro ~/.rustup -- env RUSTUP_HOME=$HOME/.rustup cargo test
...
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

- `--ro ~/.rustup` mounts the toolchains read-only at their own path.
- `cargo` resolves to `/usr/bin/cargo`, the rustup proxy on this host, through the sandbox's `PATH`.
- `env RUSTUP_HOME=…` tells the proxy where the toolchains are; your shell expands `$HOME` before
  the sandbox starts. Without it the proxy looks under `HOME=/tmp` and stops with "rustup could
  not choose a version of cargo to run".
- `target/` is written in the project directory, the one writable place.

A crate with dependencies needs them before the run, because `cargo` cannot reach a registry from
inside. Fetch them outside (`cargo fetch`), then pass `--ro ~/.cargo` and
`CARGO_HOME=$HOME/.cargo` the same way. Where the registry must be reachable, `--net` keeps the
host network.

## Any other toolchain

The pattern is the same: bind the installation with `--ro`, then set the variables it reads with
`env` in front of the command. The command line has no option to pass environment variables; the
library has `Options::env` ([Embed the library](./embed-the-library.md)).
