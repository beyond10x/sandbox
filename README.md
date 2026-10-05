# sandbox

`b10x-sandbox` respawns your shell, or one command, inside a
[bubblewrap](https://github.com/containers/bubblewrap) sandbox. The current directory is bound
writable at `/workspace`, the directories you name with `--dir` are mirrored beside it, and
everything else on the host is absent apart from `/usr`, `/bin`, `/lib` and `/lib64`, read-only.
No network unless `--net`. The namespace and mount posture is
[Substrate](https://beyond10x.github.io/substrate/)'s; this is a wrapper, not a daemon, with no
cgroup bounds, quotas or seccomp filter. The same computation is the Rust library `b10x_sandbox`.

**Status:** early, Linux only, no release yet — build from source.
**Documentation:** the site is not published yet; its pages are in [`website/docs/`](website/docs/).

| Start with | |
|---|---|
| [Getting started](website/docs/getting-started.md) | install it and watch what a sandboxed command can see |
| [What is confined](website/docs/concepts/confinement.md) | every bubblewrap argument and what it does not stop |
| [CLI reference](website/docs/reference/cli.md) | every option, generated from the clap definition |
| [Embed the library](website/docs/guides/embed-the-library.md) | `Layout`, `Confinement`, `Options` |
| [Status](website/data/status.json) | what is shipped, and the test that holds each claim |

```console
$ cargo install --locked --git https://github.com/beyond10x/sandbox b10x-sandbox
$ cd ~/work/app
$ b10x-sandbox                      # your $SHELL, cwd /workspace, no network
$ b10x-sandbox --dir ../lib         # ~/work becomes /workspace: /workspace/app and /workspace/lib
$ b10x-sandbox -- ip -brief link    # one command; only the loopback interface
$ b10x-sandbox --backend docker     # the same layout in a debian:stable-slim container
$ b10x-sandbox --dry-run            # print the bwrap argument list, one per line, and exit
```

## Build

Rust 1.97 (pinned by `rust-toolchain.toml`), Linux, `bwrap` on `PATH`; Docker for the Docker
backend and its tests.

```console
$ task check      # fmt, clippy -D warnings, tests, generated docs current
$ task install    # cargo install into ~/.cargo/bin
$ task website    # build the documentation site (Node 20 or newer)
```

The bubblewrap and Docker integration tests run only when `bwrap` or a Docker daemon is found;
otherwise each prints `<program> absent: confinement not observed` and does not claim the
confinement was verified. Changes are in [`CHANGELOG.md`](CHANGELOG.md); working on the
repository is in [`AGENTS.md`](AGENTS.md).

Apache-2.0.
