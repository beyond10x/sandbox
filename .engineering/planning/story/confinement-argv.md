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
revision: 10
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
   so the synthesized parents are not writable — **omitted when a mapping is bound at `/workspace`
   itself**, which is exactly the default layout, planned with no `--dir`
   (`Layout::binds_workspace_root()`). `--remount-ro` acts on the mount at that path and not on
   what is mounted under it, so over synthesized parents it makes the parents read-only and leaves
   the binds writable, but over the default layout it remounts the single writable bind and leaves
   the caller nothing to write. The epic's own outcome — "with no `--dir`, the current directory is
   bound writable at `/workspace`" — is what the unconditional form contradicted;
5. `--chdir <cwd's mount>`, `--clearenv`, then `--setenv` for `PATH=/usr/bin:/bin`, `HOME=/tmp`,
   `LANG` and `LC_ALL` `C.UTF-8`, `TZ=UTC`, `TERM` when `term` is set, and every `env` pair;
6. `--` followed by the command argv.

`command()` returns a `std::process::Command` for a caller that wants to wait rather than exec.
`run()` reads `/proc/sys/dev/tty/legacy_tiocsti`: when it is absent or not `0` and `allow_tiocsti`
is false, the run is refused with `LegacyTiocsti`. It then execs bubblewrap, replacing the process;
a `NotFound` from exec is `BubblewrapMissing`.

Recorded as a design change rather than a cleanup, per `AGENTS.md` invariant 4. It closes ORG-0002
of the 2026-09-15 org-state review (`atlas/reviews/org-state/2026-09-15-full-01/findings.md`
§ ORG-0002, `workspaces.md` § Sandbox), whose default probe printed `workspace-read-only`, exit 1.

## Tests

Six unit tests in `src/confinement.rs`: the exact argv for a two-directory layout with `TERM` set;
`network` dropping only `--unshare-net` and roots preceding the workspace; the empty-command and
bad-root refusals; the sysctl reading (absent, `0`, `1`); the default layout — built by
`Layout::plan` with no dirs, not by hand — binding the working directory at `/workspace` and
carrying no `--remount-ro`; and a mirrored layout still carrying `--remount-ro /workspace` after
every bind.

Two integration tests in `tests/bubblewrap.rs`, each a no-op printing `bwrap absent: confinement
not observed` when `bwrap` is not on `PATH`:

- two directories under one temporary root, observing cwd at the mapped path, only the mapped
  children visible at each level, a relative write between the two directories succeeding on the
  host, a write into a synthesized parent and into `/workspace` refused, a sibling file absent, and
  `HOME=/tmp`;
- the default layout — no `--dir` — observing `test -w /workspace` true, a file written, read back
  and removed under `/workspace`, a second file that survives and is then found in the host
  directory, an undeclared sibling directory absent, a `--ro` root readable, and a write into that
  root refused. This is the closing proof ORG-0002 asks for; there are no synthesized parents in
  this layout, so that denial stays with the mirrored test above.

## Acceptance

On a machine with `bwrap`, `cargo test --locked` exits 0 with every test named in Tests listed
`ok`.

## Scope

`src/confinement.rs`, `tests/bubblewrap.rs`, and one `pub use` line in `src/lib.rs`.
