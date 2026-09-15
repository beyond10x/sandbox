---
format: aep.planning-md/1
id: epic:docker-backend
kind: epic
status: implemented
title: A Docker backend beside bubblewrap
summary: b10x-sandbox --backend docker runs the same mirrored /workspace layout in a container; bubblewrap stays the default.
relations:
- informed_by: epic:sandbox-shell
revision: 6
---
## Context

`epic:sandbox-shell` delivered the layout and a bubblewrap runner. A probe on 2026-09-13 (Docker
29.7.2, `debian:stable-slim`) showed Docker reproduces the same eight observations the bubblewrap
integration test makes — cwd at the mapped path, only mapped children visible, a relative write
between mapped directories reaching the host, writes into synthesized parents and into
`/workspace` refused, a sibling file absent, `HOME=/tmp` — with a read-only tmpfs at
`/workspace` (`tmpfs-mode=0555`) and the container running as the caller's uid and gid.

## Outcome

- `b10x-sandbox --backend docker` runs the command in a container from `--image` (default
  `debian:stable-slim`) with the same `Layout`: one writable bind per mapping, one read-only bind
  per `--ro` root at its own path, `-w` at the mapped working directory.
- `--backend bubblewrap` stays the default; omitting the flag changes nothing from
  `epic:sandbox-shell`.
- The Docker posture, as close to the bubblewrap one as `docker run` allows: `--rm`, `--user
  <uid>:<gid>` of the caller, `--network none` unless `--net`, `--cap-drop ALL`, `--security-opt
  no-new-privileges`, `--read-only` image root, `--tmpfs /tmp`, tmpfs `/workspace` at mode 0555,
  `-i`, and `-t` only when the caller's stdin is a terminal.
- The environment is set with `-e` for the same baseline the bubblewrap backend sets; Docker has no
  clear-environment step, so the image's own variables remain.
- The default command under Docker is `/bin/sh`, because the image and not the host decides which
  shells exist; a trailing `-- CMD...` runs that instead.
- A missing `docker` binary is a named refusal, like a missing `bwrap`.
- The library exposes the backend as `Backend::{Bubblewrap, Docker { image, tty }}` on `Options`,
  and `Confinement::program()` names the binary `run()` will exec.

## Boundaries

- No daemon management: Docker must already be running; a failed `docker run` is reported by exit
  code, not diagnosed.
- No image building and no pulling logic of its own; `docker run` pulls on first use as it always
  does.
- The TIOCSTI check applies only to bubblewrap: Docker allocates its own pseudo-terminal, so the
  container never holds the caller's terminal.
- No entity is introduced.

## Done when

On a machine with a running Docker daemon, `cargo test --locked` exits 0 with the
`tests/docker.rs` test and `tests/cli.rs`'s
`docker_backend_dry_run_prints_docker_and_the_mirrored_volumes` both listed `ok` in its output.
