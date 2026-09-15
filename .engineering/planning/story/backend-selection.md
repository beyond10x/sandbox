---
format: aep.planning-md/1
id: story:backend-selection
kind: story
status: implemented
title: Select the backend from the library and the CLI
relations:
- decomposes: epic:docker-backend
- depends_on: story:docker-argv
scope:
- confidence: cited
  path: README.md
- confidence: cited
  path: src/confinement.rs
- confidence: cited
  path: src/lib.rs
- confidence: cited
  path: src/main.rs
- confidence: cited
  path: tests/cli.rs
revision: 9
---
## Context

`Confinement` currently knows only bubblewrap. This story makes the backend a value on `Options`,
dispatches `argv()` and `run()` on it, and exposes the choice on the command line. It changes the
files `epic:sandbox-shell` created; the bubblewrap behaviour and its tests are unchanged.

## Behaviour

- `confinement::Backend { Bubblewrap, Docker { image: String, tty: bool } }`, `Default` is
  `Bubblewrap`; `Options.backend` replaces nothing and `Options.bubblewrap` (the binary path) stays.
- `Confinement::program()` returns `bwrap` or `docker`; the former `bubblewrap()` accessor is
  removed. `argv()` dispatches to the existing bubblewrap list or `docker::argv`. `run()` performs
  the TIOCSTI check only for bubblewrap and reports a `NotFound` exec as `ProgramMissing(path)`.
- `src/lib.rs` declares `pub mod docker;` and re-exports `Backend`.
- CLI: `--backend <bubblewrap|docker>` (default `bubblewrap`), `--image <NAME>` (default
  `debian:stable-slim`, meaningful only with `docker`); under Docker the default command is
  `/bin/sh`, `tty` is `stdin.is_terminal()`, and `--dry-run` prints `docker` then the argv.
- `README.md` documents the backend, its posture table and the differences from bubblewrap.

## Tests

`tests/cli.rs` gains one test: `--backend docker --dry-run --dir ../../y/z/b` from `x/a` prints
`docker` on the first line, `-w` followed by `/workspace/x/a`, and two `-v` values naming both
directories under `/workspace`.

## Acceptance

`cargo test --locked` reports every test passed with 0 failed, including the new CLI test.

## Scope

`src/confinement.rs`, `src/main.rs`, `src/lib.rs`, `tests/cli.rs`, `README.md`.
