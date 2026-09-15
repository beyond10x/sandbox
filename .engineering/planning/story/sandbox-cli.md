---
format: aep.planning-md/1
id: story:sandbox-cli
kind: story
status: implemented
title: The b10x-sandbox command
relations:
- decomposes: epic:sandbox-shell
- depends_on: story:confinement-argv
scope:
- confidence: cited
  path: src/main.rs
- confidence: cited
  path: tests/cli.rs
revision: 7
---
## Context

The command is the operator-facing surface. It is clap derive over the library and owns no logic
of its own beyond turning flags into an `Options`. It replaces the placeholder `src/main.rs` from
`task:crate-scaffold`.

## Behaviour

```
b10x-sandbox [--dir <PATH>]... [--ro <PATH>]... [--net] [--allow-tiocsti] [--dry-run] [-- <CMD>...]
```

- `--dir` is repeatable; a relative path is joined onto the caller's cwd and `.`/`..` resolved
  lexically, and the library's canonical check still refuses a symlink;
- `--ro` is repeatable and binds a host directory read-only at its own absolute path, never
  writable;
- `--net` keeps the host network namespace; the default is no network;
- no `CMD` runs `$SHELL`, and `/bin/sh` when `SHELL` is unset; `TERM` is copied from the caller;
- `--dry-run` prints the bubblewrap binary and then its argv one argument per line, exit 0;
- every library refusal, and a missing `bwrap`, is printed as `b10x-sandbox: <message>` on stderr
  with exit code 2.

## Tests

Two integration tests in `tests/cli.rs`: `--dry-run --dir ../../y/z/b` from `x/a` under one
temporary root prints `--chdir /workspace/x/a` and two `--bind` lines naming both directories
under `/workspace`; `--dir does-not-exist` exits 2 with `b10x-sandbox: ` and `not an existing
directory` on stderr.

## Acceptance

`cargo test --locked --test cli` reports 2 passed and 0 failed.

## Scope

`src/main.rs` and `tests/cli.rs`.
