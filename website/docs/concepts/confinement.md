---
title: What is confined
sidebar_position: 1
description: The bubblewrap argument set b10x-sandbox passes, where it comes from, how it differs from Substrate's, and what it does not stop.
lede: The isolation set is Substrate's bubblewrap namespace and mount list with three named differences; it confines what a process can see, not how much it can use.
source: src/confinement.rs, src/docker.rs, tests/substrate_mirror.rs, tests/bubblewrap.rs, tests/backend_refusal.rs; Substrate crates/substrate-host/src/process.rs at 0.7.0
---

# What is confined

`b10x-sandbox` builds one `bwrap` argument list and replaces itself with `bwrap`. Every argument
is fixed except the ones its options name. [Argument lists](../reference/argv.md) shows the full
list, generated from the code.

## The bubblewrap set

| What | Arguments |
|---|---|
| own user namespace, no nested one | `--unshare-user --disable-userns` |
| own ipc, pid and uts namespaces; own network unless `--net` | `--unshare-ipc --unshare-pid --unshare-uts --unshare-net` |
| dies with the process that started it | `--die-with-parent` |
| base system, read-only | `--ro-bind /usr /usr`, `--ro-bind-try` for `/bin`, `/lib`, `/lib64` |
| private proc, dev and tmp | `--proc /proc --dev /dev --tmpfs /tmp` |
| each `--ro` directory, read-only at its own path | `--ro-bind <dir> <dir>` |
| the workspace | `--tmpfs /workspace`, one `--bind` per directory, then `--remount-ro /workspace`; in the default layout one `--bind <cwd> /workspace` and no remount |
| working directory | `--chdir` to the working directory's place under `/workspace` |
| environment | `--clearenv`, then `PATH=/usr/bin:/bin`, `HOME=/tmp`, `LANG` and `LC_ALL` `C.UTF-8`, `TZ=UTC`, and the caller's `TERM` |

Nothing else from the host is mounted: no `/etc`, no `/home`, no `/var`, no `/sys`, no sockets.
A program that needs `/etc` (DNS resolution, TLS certificates, user names) does not find it.

## Where it comes from

The list is copied from Substrate's host driver (`crates/substrate-host/src/process.rs` at
Substrate 0.7.0). `tests/substrate_mirror.rs` writes Substrate's arguments out as the oracle and
fails when the list here drops one without the module documentation naming it. The three
differences are deliberate:

| Difference | Why |
|---|---|
| no `--new-session` | an interactive shell needs its controlling terminal; [the terminal guard](#the-terminal) stands in for it |
| no seccomp filter | Substrate's socket-family filter is not carried |
| `/workspace` is a tmpfs with one bind per directory, remounted read-only | Substrate binds one workspace directory; here several are mirrored ([The workspace layout](./workspace-layout.md)) |

## The terminal

`--new-session` stops a sandboxed process from injecting keystrokes into your terminal with the
`TIOCSTI` ioctl. Without it, that hazard is closed by the kernel when `dev.tty.legacy_tiocsti` is
`0` (kernels 6.2 and later can set it). Before running, `b10x-sandbox` reads
`/proc/sys/dev/tty/legacy_tiocsti` and refuses when it is `1`, or when it is missing, which means a
kernel where the hazard is always present:

```text
b10x-sandbox: /proc/sys/dev/tty/legacy_tiocsti is 1: a sandboxed process could inject terminal input; set it to 0 or pass --allow-tiocsti
```

`--allow-tiocsti` runs anyway. The Docker backend skips the check: Docker allocates its own
pseudo-terminal.

## Refusals

A refusal exits 2 with `b10x-sandbox: <reason>` and runs nothing:

- an input path that is relative after resolution, not canonical, not a directory, listed twice or
  nested in another ([The workspace layout](./workspace-layout.md#refusals));
- a backend program that is not on `PATH`: `b10x-sandbox: bwrap not found`, or
  `b10x-sandbox: docker not found` under `--backend docker`.

A missing backend never falls back to running the command unconfined or on the other backend;
`tests/backend_refusal.rs` runs the binary with an empty `PATH` and checks that the command did
not run.

## What it does not stop

- **Use of resources.** No cgroup, so no CPU, memory, process-count or disk bounds; a fork bomb or
  a full disk inside `/workspace` is the host's problem. Substrate bounds these.
- **System calls.** No seccomp filter; the process may call anything the kernel allows an
  unprivileged user in its own namespaces.
- **What you bind.** A `--dir` is writable and a `--ro` is readable. Binding a directory that
  holds credentials hands them to the process.
- **The network, with `--net`.** The host namespace is shared, with no destination filter.
- **A record.** Nothing is logged or kept; there is no daemon.

## Docker

`--backend docker` builds the closest `docker run` list instead: `--user <uid>:<gid>`,
`--network none` unless `--net`, `--cap-drop ALL`, `--security-opt no-new-privileges`,
`--read-only` with a tmpfs `/tmp`, the same volumes and the same environment baseline.
`tests/substrate_mirror.rs` checks that every Substrate argument with a Docker equivalent has
one. [Use the Docker backend](../guides/use-the-docker-backend.md) lists where the two differ.
