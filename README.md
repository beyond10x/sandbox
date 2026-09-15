# sandbox

`b10x-sandbox` respawns your shell inside a [bubblewrap](https://github.com/containers/bubblewrap)
sandbox. The current directory is bound writable at `/workspace` — a file created, read and removed
there is created, read and removed in that host directory — and everything else on the host is
absent, apart from `/usr`, `/bin`, `/lib` and `/lib64` read-only. With `--dir`, `/workspace` becomes
a read-only root holding one writable bind per named directory, as the next section describes.

```console
$ cd ~/work/app
$ b10x-sandbox                      # your $SHELL, cwd /workspace, no network
$ b10x-sandbox --dir ../lib         # ~/work becomes /workspace: /workspace/app and /workspace/lib,
                                    # nothing else from ~/work is visible; shell starts in /workspace/app
$ b10x-sandbox --ro ~/.cargo --ro ~/.rustup -- cargo test   # toolchain homes read-only, one command
$ b10x-sandbox --net                # keep the host network namespace
$ b10x-sandbox --dry-run            # print the bwrap argv, one argument per line, and exit
$ b10x-sandbox --backend docker     # same layout in a debian:stable-slim container, /bin/sh
$ b10x-sandbox --backend docker --image rust:1.97-slim -- cargo test
```

With `--dir`, the common ancestor of the current directory and every `--dir` becomes `/workspace`.
Each directory is bound writable at its path relative to that ancestor; the directories in between
are empty, read-only, and vanish with the sandbox. Relative references between the mapped
directories (`../lib`) resolve exactly as they do on the host.

Without `--dir` there is nothing in between: the current directory is the ancestor and is bound at
`/workspace` itself, so `/workspace` is the writable directory rather than a read-only root over
one. The step that makes the synthesized parents read-only is therefore not taken in that layout —
it acts on the mount at `/workspace`, which in the default layout is the only bind there is.

## Confinement

The namespace and mount posture is copied from [substrate](https://github.com/beyond10x/substrate)
(`crates/substrate-host/src/process.rs` at 0.7.0), as a list of `bwrap` arguments:

| what | argument |
|---|---|
| own user namespace, no nested one | `--unshare-user --disable-userns` |
| ipc, pid, uts unshared; net unless `--net` | `--unshare-ipc --unshare-pid --unshare-uts --unshare-net` |
| base system read-only | `--ro-bind /usr`, `--ro-bind-try /bin /lib /lib64` |
| private proc, dev, tmp | `--proc /proc --dev /dev --tmpfs /tmp` |
| workspace | `--tmpfs /workspace`, one `--bind` per directory, then `--remount-ro /workspace` — omitted when a directory is bound at `/workspace` itself, which is the default layout |
| environment | `--clearenv`, then `PATH HOME LANG LC_ALL TZ TERM` |
| lifetime | `--die-with-parent` |

What it does not do: no cgroup bounds, no quotas, no seccomp filter, no ledger. Those are
substrate's, and this tool is a wrapper, not a daemon.

## Docker backend

`--backend docker` runs the same layout through `docker run`. Bubblewrap stays the default.

| what | argument |
|---|---|
| one-shot, interactive, terminal only when stdin is one | `run --rm -i [-t]` |
| the caller's identity, so the binds are owned by the process inside | `--user <uid>:<gid>` |
| no network unless `--net` | `--network none` |
| no capabilities, no privilege escalation, image root read-only, private tmp | `--cap-drop ALL --security-opt no-new-privileges --read-only --tmpfs /tmp` |
| workspace | `--mount type=tmpfs,destination=/workspace,tmpfs-mode=0555` — omitted in the default layout, where Docker refuses a tmpfs and a volume at one destination — one `-v` per directory, `-v <root>:<root>:ro` per `--ro` |
| environment | `-e` for the same `PATH HOME LANG LC_ALL TZ TERM` baseline, on top of the image's own |

Differences from bubblewrap: the container runs as your uid rather than in a user namespace; the
synthesized parents are read-only through the tmpfs mode, so a root user inside the image would
not be stopped by it; the environment is not cleared; the default command is `/bin/sh` because the
image, not the host, decides which shells exist. Docker must already be running; a missing
`docker` binary is refused by name, a failed `docker run` is reported by its exit code.

`--new-session` is not passed, because an interactive shell needs its controlling terminal. The
hazard that flag closes (a sandboxed process injecting input into the terminal with `TIOCSTI`) is
closed on kernels 6.2 and later by `dev.tty.legacy_tiocsti = 0`; the tool reads that sysctl and
refuses to run when it is `1` unless `--allow-tiocsti` is given.

## Library

The same computation is a Rust library, `b10x_sandbox`:

```rust
use b10x_sandbox::{Confinement, Layout, Options};

let layout = Layout::plan(&std::env::current_dir()?, &["/home/me/work/lib".into()])?;
let confinement = Confinement::new(layout, Options::default(), ["/bin/sh"])?;
for argument in confinement.argv()? { println!("{}", argument.to_string_lossy()); }
confinement.run()?; // replaces the current process; Options { backend: Backend::Docker {..}, .. } for Docker
```

## Build

Rust 1.97 (pinned by `rust-toolchain.toml`), Linux, `bwrap` on `PATH`.

```console
$ task check      # fmt, clippy -D warnings, tests
$ task install    # cargo install into ~/.cargo/bin
```

The bubblewrap and Docker integration tests run only when `bwrap` or a Docker daemon is found;
otherwise each prints `<program> absent: confinement not observed` and does not claim the
confinement was verified.

Apache-2.0.
