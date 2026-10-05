---
title: What is Sandbox?
slug: /
sidebar_position: 1
description: A Rust command line and library that respawns a shell or one command inside a bubblewrap sandbox that sees only the directories you name.
lede: b10x-sandbox confines one process on a Linux machine to the directories you name, with Substrate's namespace and mount posture and nothing more.
source: src/lib.rs, src/confinement.rs, src/docker.rs, src/layout.rs, tests/
---

# What is Sandbox?

`b10x-sandbox` respawns your shell, or one command, inside a
[bubblewrap](https://github.com/containers/bubblewrap) sandbox. The current directory is bound
writable at `/workspace`. Everything else on the host is absent, apart from `/usr`, `/bin`, `/lib`
and `/lib64`, read-only. There is no network unless you pass `--net`.

```console
$ cd ~/work/app
$ b10x-sandbox                                   # your $SHELL, in /workspace, no network
$ b10x-sandbox --dir ../lib                      # ~/work becomes /workspace, holding app and lib
$ b10x-sandbox -- ip -brief link                 # one command: only the loopback interface
$ b10x-sandbox --backend docker                  # the same layout through docker run
$ b10x-sandbox --dry-run                         # print the bwrap argument list and exit
```

The same computation is a Rust library, `b10x_sandbox`: `Layout::plan` works out where each
directory lands, and `Confinement` turns that into the exact `bwrap` or `docker run` argument list
and runs it.

## What it is not

| It is not | Because | Use instead |
|---|---|---|
| a resource limiter | no cgroup bounds, no CPU, memory, process or disk quotas | [Substrate](https://beyond10x.github.io/substrate/) ([GitHub](https://github.com/beyond10x/substrate)) |
| a syscall filter | no seccomp filter; Substrate's socket-family filter is not carried | Substrate |
| a service | no daemon, no API, no record of what ran | Substrate |
| portable | Linux only; bubblewrap or a running Docker daemon | — |
| a credential boundary for what you bind | a `--dir` is writable and a `--ro` is readable by the sandboxed process | name less |

## Where it sits

- **[Substrate](https://beyond10x.github.io/substrate/)
  ([GitHub](https://github.com/beyond10x/substrate))** is a governed execution service: it admits,
  bounds and records operations on one Linux host. Sandbox copies the bubblewrap namespace and
  mount argument set from Substrate's host driver and nothing else, and a test in this repository
  holds the two lists together, difference by difference
  ([What is confined](./concepts/confinement.md)).

## Read next

- [Getting started](./getting-started.md): install it and watch what the sandbox can and cannot see.
- [What is confined](./concepts/confinement.md): every argument, and what it does not stop.
- [The workspace layout](./concepts/workspace-layout.md): how `--dir` mirrors directories.
- [Status](/docs/status): what is shipped, and the test that holds each claim.
