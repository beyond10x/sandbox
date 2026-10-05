---
title: Use the Docker backend
sidebar_position: 2
description: Run the same layout through docker run instead of bubblewrap, and where the two backends differ.
lede: --backend docker runs the same mirrored layout in a container as your own uid, with no capabilities, no network and a read-only image root.
source: src/docker.rs, tests/docker.rs, tests/substrate_mirror.rs; run with b10x-sandbox 0.1.0 and Docker 29.7.2
---

# Use the Docker backend

Bubblewrap stays the default. `--backend docker` needs a running Docker daemon and builds a
`docker run` list from the same layout. The default image is `debian:stable-slim` and the default
command `/bin/sh`, because the image, not the host, decides which shells exist.

```console
$ cd ~/.cache/sandbox-demo/app
$ b10x-sandbox --backend docker -- sh -c 'id -u; pwd; cat /etc/os-release | head -1'
1000
/workspace
PRETTY_NAME="Debian GNU/Linux 13 (trixie)"
```

The process runs as your uid, so files it writes in `/workspace` are yours on the host. The
mirrored layout works the same way:

```console
$ b10x-sandbox --backend docker --dir ../lib -- sh -c 'pwd; ls /workspace; cat ../lib/note.txt; touch /workspace/new'
/workspace/app
app
lib
hello from lib
touch: cannot touch '/workspace/new': Permission denied
```

`--image <NAME>` picks another image, such as one that carries your toolchain.

## How it differs from bubblewrap

| | Bubblewrap | Docker |
|---|---|---|
| identity | a user namespace | `--user <uid>:<gid>` |
| what is visible | only `/usr`, `/bin`, `/lib`, `/lib64` from the host | the image's own filesystem, read-only (`--read-only`), plus `/tmp` |
| synthesized parents under `/workspace` | a tmpfs remounted read-only | a tmpfs at mode `0555`, which a root user inside the image would not be stopped by |
| environment | cleared, then the baseline | the image's own variables, with the baseline set over them |
| capabilities | none, through the user namespace | `--cap-drop ALL --security-opt no-new-privileges` |
| seccomp | none | Docker's default profile |
| terminal | refuses when `dev.tty.legacy_tiocsti` is 1 | no check; Docker allocates its own pseudo-terminal, `-t` only when stdin is one |

A missing `docker` binary is refused as `b10x-sandbox: docker not found` and never falls back to
bubblewrap. A `docker run` that fails reports its own exit status. The full list is in
[Argument lists](../reference/argv.md#docker-mirrored-layout).
