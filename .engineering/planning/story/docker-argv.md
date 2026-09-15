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
revision: 9
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
5. `--mount type=tmpfs,destination=/workspace,tmpfs-mode=0555`;
6. `-v <root>:<root>:ro` per read-only root, then `-v <host>:<mount>` per mapping;
7. `-w <cwd mount>`;
8. `-e NAME=VALUE` for `PATH=/usr/bin:/bin`, `HOME=/tmp`, `LANG=C.UTF-8`, `LC_ALL=C.UTF-8`,
   `TZ=UTC`, `TERM` when `options.term` is set, and every `options.env` pair;
9. the image, then the command argv.

`identity` is a parameter so the unit test can fix it; `docker::caller_identity()`, read from the
owner of `/proc/self`, is what `Confinement` passes.

## Tests

One unit test in `src/docker.rs` asserting the exact argv for a two-directory layout with a
read-only root, `TERM` set and `tty` true, and one asserting `caller_identity()` matches the
owner of a file this process creates. One integration test in `tests/docker.rs` that, when
`docker version` succeeds, calls `docker::argv` directly (not `Confinement`, which learns about
Docker only in `story:backend-selection`) and runs `docker` with it: `/bin/sh` in
`debian:stable-slim` over the same temporary tree the bubblewrap test uses, asserting the same
eight observation lines; otherwise it prints `docker absent: confinement not observed` and returns.

## Acceptance

On a machine with Docker running, `cargo test --locked` exits 0 with
`docker::tests::argv_for_a_two_directory_layout_with_a_root_is_exact` and the `tests/docker.rs`
test both listed `ok` in its output.

## Scope

`src/docker.rs` and `tests/docker.rs`.
