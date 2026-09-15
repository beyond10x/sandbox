---
format: aep.planning-md/1
id: story:docker-argv
kind: story
status: implemented
title: Build the docker run argv from a layout
relations:
- decomposes: epic:docker-backend
scope:
- confidence: cited
  path: src/docker.rs
- confidence: cited
  path: tests/docker.rs
revision: 11
---
## Context

The Docker invocation is a pure function of the same inputs the bubblewrap one takes, so it lives
in its own module and can be tested for exact bytes without a daemon.

## Behaviour

`docker::argv(layout, options, image, tty, identity, command)` returns the complete `docker`
argument list:

1. `run --rm -i`, plus `-t` when `tty` is set;
2. `--user <uid>:<gid>` from `identity`;
3. `--network none` unless `options.network`;
4. `--cap-drop ALL --security-opt no-new-privileges --read-only --tmpfs /tmp`;
5. `--mount type=tmpfs,destination=/workspace,tmpfs-mode=0555` — **omitted when a mapping is bound
   at `/workspace` itself**, which is the default layout, planned with no `--dir`
   (`Layout::binds_workspace_root()`). The tmpfs exists to make the synthesized parents read-only
   through its `0555` mode; the default layout has none, and Docker refuses a tmpfs and a volume at
   one destination ("Duplicate mount point"), so emitting it unconditionally does not start a
   read-only run, it starts no run at all;
6. `-v <root>:<root>:ro` per read-only root, then `-v <host>:<mount>` per mapping;
7. `-w <cwd mount>`;
8. `-e NAME=VALUE` for `PATH=/usr/bin:/bin`, `HOME=/tmp`, `LANG=C.UTF-8`, `LC_ALL=C.UTF-8`,
   `TZ=UTC`, `TERM` when `options.term` is set, and every `options.env` pair;
9. the image, then the command argv.

`identity` is a parameter so the unit test can fix it; `docker::caller_identity()`, read from the
owner of `/proc/self`, is what `Confinement` passes.

Step 5's condition is recorded as a design change rather than a cleanup, per `AGENTS.md`
invariant 4. It is the Docker half of ORG-0002 of the 2026-09-15 org-state review, which asked for
Docker to be checked separately because its default layout had not been exercised
(`atlas/reviews/org-state/2026-09-15-full-01/workspaces.md` § Sandbox).

## Tests

Three unit tests in `src/docker.rs`: the exact argv for a two-directory layout with a read-only
root, `TERM` set and `tty` true; `caller_identity()` matching the owner of a file this process
creates; and the default layout — built by `Layout::plan` with no dirs, not by hand — carrying no
`destination=/workspace` tmpfs, the working directory as a writable `-v ...:/workspace`, and
`-w /workspace`.

Two integration tests in `tests/docker.rs`, each a no-op printing `docker absent: confinement not
observed` when `docker version` fails. Both call `docker::argv` directly, not `Confinement`, which
learns about Docker only in `story:backend-selection`, and run `/bin/sh` in `debian:stable-slim`:

- the two-directory layout over the same temporary tree the bubblewrap test uses, asserting the
  same eight observation lines;
- the default layout, asserting the same ten lines its bubblewrap counterpart does: `test -w
  /workspace` true, a file written, read back and removed, a second file found afterwards in the
  host directory, an undeclared sibling absent, a `--ro` root readable and not writable.

## Acceptance

On a machine with Docker running, `cargo test --locked` exits 0 with
`docker::tests::argv_for_a_two_directory_layout_with_a_root_is_exact` and the `tests/docker.rs`
test both listed `ok` in its output.

## Scope

`src/docker.rs` and `tests/docker.rs`.
