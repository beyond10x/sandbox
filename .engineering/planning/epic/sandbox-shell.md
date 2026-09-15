---
format: aep.planning-md/1
id: epic:sandbox-shell
kind: epic
status: implemented
title: Respawn the current shell inside a bubblewrap sandbox
summary: 'b10x-sandbox: a Rust library and CLI that runs the caller''s shell under substrate''s confinement posture with chosen host directories mirrored writable under /workspace.'
revision: 5
---
## Context

Substrate confines a process with bubblewrap but refuses writable host directories by design
(substrate `adr/0010`, `adr/0023`). An operator who wants their own shell, in their own project
directory, under the same namespace and mount posture has no tool. A probe on 2026-09-13
(bwrap 0.12.0, kernel 6.18.49) showed bubblewrap itself can mirror two arbitrary host directories
under a tmpfs `/workspace` with empty synthesized parents, so the tool is a wrapper, not a daemon.

## Outcome

`b10x-sandbox` replaces the current process with a sandboxed process:

- with no `--dir`, the current directory is bound writable at `/workspace`;
- with `--dir A --dir B`, the common ancestor of pwd and every dir becomes `/workspace`, each
  directory is bound writable at `/workspace/<path relative to that ancestor>`, the parents in
  between are empty read-only tmpfs directories, and the process starts at pwd's mapped path.
  Directories under different top-level directories mirror from `/`: only the named directories
  are bound, so nothing else becomes visible;
- with no command, the process is `$SHELL` (`/bin/sh` when unset); a trailing `-- CMD...` runs
  that command instead, so `b10x-sandbox -- cargo test` is one confined run;
- `--ro PATH` binds a host directory read-only at its own absolute path, for a toolchain home such
  as `~/.cargo` (substrate `adr/0010` is the same mechanism, with the same never-writable rule);
- the namespace and mount posture is substrate's (`crates/substrate-host/src/process.rs:1911-1937`
  at substrate 0.7.0): user namespace with `--disable-userns`, ipc/pid/uts unshared, net unshared
  unless `--net` is given, `/usr` `/bin` `/lib` `/lib64` read-only, private `/proc` `/dev` `/tmp`,
  cleared environment with a fixed baseline;
- `--dry-run` prints the bubblewrap argv instead of running it;
- the library `b10x_sandbox` exposes the same computation (`Layout`, `Confinement`, `Options`) so
  another Rust program can plan and run a sandbox, including an `env` option the CLI does not
  expose.

## Boundaries

- Linux and bubblewrap only. Absence of `bwrap` is a named refusal, never a fallback.
- No daemon, no ledger, no cgroup bounds, no quotas. Those are substrate's and are not copied.
- No seccomp filter in the first release; substrate's socket-family filter is a later milestone.
- No `--new-session`: an interactive shell needs its controlling terminal. The TIOCSTI hazard that
  flag closes is closed on kernels 6.2 and later by `dev.tty.legacy_tiocsti=0`; the tool reads that
  sysctl and refuses when it is not `0` or is absent, unless `--allow-tiocsti` is given.
- No entity is introduced: the tool persists nothing, so no ESS document is drafted.

## Done when

Every decomposed item is implemented and `task check` (fmt, clippy with `-D warnings`, tests) is
green on a machine with bubblewrap.
