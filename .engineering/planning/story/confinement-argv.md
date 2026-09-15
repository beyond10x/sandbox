---
format: aep.planning-md/1
id: story:confinement-argv
kind: story
status: implemented
title: Build the bubblewrap argv from a layout
relations:
- decomposes: epic:sandbox-shell
- depends_on: story:workspace-layout
scope:
- confidence: cited
  path: src/confinement.rs
- confidence: cited
  path: src/lib.rs
- confidence: cited
  path: tests/bubblewrap.rs
revision: 8
---
## Context

The isolation set is substrate's, copied as a list of bubblewrap arguments so a reader can compare
it line by line with `crates/substrate-host/src/process.rs:1911-1937` at substrate 0.7.0. This
story replaces the placeholder `src/confinement.rs` from `task:crate-scaffold` and adds its
re-export line to `src/lib.rs`.

## Behaviour

`Confinement::new(layout, options, command)` takes a `Layout`, an `Options { network,
allow_tiocsti, read_only_roots, term, env, bubblewrap }` and a command argv. It refuses an empty
command and validates every read-only root with `layout::canonical_directory`. `argv()` is the
complete `bwrap` argument list:

1. `--unshare-user --disable-userns --unshare-ipc --unshare-pid --unshare-uts`, plus
   `--unshare-net` unless `network` is set, then `--die-with-parent`;
2. `--ro-bind /usr /usr`, `--ro-bind-try` for `/bin` `/lib` `/lib64`, `--proc /proc`, `--dev /dev`,
   `--tmpfs /tmp`;
3. `--ro-bind <root> <root>` for every read-only root;
4. `--tmpfs /workspace`, then `--bind <host> <mount>` per mapping, then `--remount-ro /workspace`
   so the synthesized parents are not writable;
5. `--chdir <cwd's mount>`, `--clearenv`, then `--setenv` for `PATH=/usr/bin:/bin`, `HOME=/tmp`,
   `LANG` and `LC_ALL` `C.UTF-8`, `TZ=UTC`, `TERM` when `term` is set, and every `env` pair;
6. `--` followed by the command argv.

`command()` returns a `std::process::Command` for a caller that wants to wait rather than exec.
`run()` reads `/proc/sys/dev/tty/legacy_tiocsti`: when it is absent or not `0` and `allow_tiocsti`
is false, the run is refused with `LegacyTiocsti`. It then execs bubblewrap, replacing the process;
a `NotFound` from exec is `BubblewrapMissing`.

## Tests

Four unit tests in `src/confinement.rs`: the exact argv for a two-directory layout with `TERM`
set; `network` dropping only `--unshare-net` and roots preceding the workspace; the empty-command
and bad-root refusals; the sysctl reading (absent, `0`, `1`). One integration test in
`tests/bubblewrap.rs` that, with `bwrap` on `PATH`, runs `/bin/sh` through `command()` over two
directories under one temporary root and observes: cwd at the mapped path, only the mapped
children visible at each level, a relative write between the two directories succeeding on the
host, a write into a synthesized parent and into `/workspace` refused, a sibling file absent, and
`HOME=/tmp`. Without `bwrap` it prints `bwrap absent: confinement not observed` and returns.

## Acceptance

On a machine with `bwrap`, `cargo test --locked` exits 0 with every test named in Tests listed
`ok`.

## Scope

`src/confinement.rs`, `tests/bubblewrap.rs`, and one `pub use` line in `src/lib.rs`.
