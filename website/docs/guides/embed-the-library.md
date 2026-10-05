---
title: Embed the library
sidebar_position: 3
description: Use the b10x_sandbox crate to compute a layout, build the argument list and run a command in a Rust program.
lede: Layout::plan computes where directories land, Confinement::new validates the rest, and the caller either replaces its process with run or waits on command.
source: src/lib.rs, src/layout.rs, src/confinement.rs; the program below was built against the Git revision it names and run with bubblewrap 0.12.0
---

# Embed the library

The command line is a thin layer over `b10x_sandbox`. The crate is not on a registry; depend on it
by Git, at a full revision:

```toml
[dependencies]
b10x-sandbox = { git = "https://github.com/beyond10x/sandbox", rev = "7a33b7b0580600db3cf8e4d7d63f0f9910d9859d" }
```

## Run a command and wait for it

```rust
use b10x_sandbox::{Confinement, Layout, Options};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bind the working directory writable at /workspace, nothing else from the host.
    let cwd = std::env::current_dir()?;
    let layout = Layout::plan(&cwd, &[])?;
    let confinement = Confinement::new(layout, Options::default(), ["sh", "-c", "pwd; ls /"])?;

    // Wait for it rather than replacing this process.
    let status = confinement.command()?.status()?;
    println!("sandboxed command exited with {status}");
    Ok(())
}
```

```console
$ cargo run -q
/workspace
bin
dev
lib
lib64
proc
tmp
usr
workspace
sandboxed command exited with exit status: 0
```

## The API

| Item | What it does |
|---|---|
| `Layout::plan(cwd, dirs)` | the mirrored layout, or a `LayoutError` naming the first input it refuses ([The workspace layout](../concepts/workspace-layout.md)) |
| `Options` | `backend`, `network`, `allow_tiocsti`, `read_only_roots`, `term`, `env` (pairs set after the baseline, able to override it) and the `bubblewrap` binary; `Options::default()` is bubblewrap, no network |
| `Backend::Docker { image, tty }` | the Docker backend |
| `Confinement::new(layout, options, command)` | validates the read-only roots and that the command is not empty; touches nothing |
| `confinement.program()`, `confinement.argv()` | the program and its full argument list, as `--dry-run` prints them |
| `confinement.command()` | a `std::process::Command` to spawn and wait on |
| `confinement.run()` | checks the terminal hazard, then replaces the current process; returns only on failure |

`command()` does not read `dev.tty.legacy_tiocsti`; only `run()` refuses on it. A caller that
spawns with `command()` and hands the child its terminal makes that check itself, or passes no
terminal. [What is confined](../concepts/confinement.md#the-terminal) explains the hazard.
