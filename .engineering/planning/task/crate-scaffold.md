---
format: aep.planning-md/1
id: task:crate-scaffold
kind: task
status: implemented
title: Scaffold the b10x-sandbox crate
relations:
- implements: epic:sandbox-shell
revision: 5
---
## Context

The stories under `epic:sandbox-shell` each own one source file. The files they share — the
manifest, the module list in `src/lib.rs`, the toolchain pin, the task runner — are created once
here. Three source files are created here as placeholders and then **replaced** by the story
named beside each, which is the one deliberate overlap in this set: the placeholder exists only so
the crate compiles before any story lands.

## Work

- `Cargo.toml`: one package `b10x-sandbox`, edition 2024, `rust-version = "1.97"`, `publish = false`,
  `[lib]` and `[[bin]] name = "b10x-sandbox"`, dependencies `clap` (derive) and `thiserror`,
  dev-dependency `tempfile`, clippy lints `all` and `pedantic` at `deny`.
- `rust-toolchain.toml` pinning 1.97 with rustfmt and clippy; `rustfmt.toml` at `max_width = 100`.
- `src/lib.rs` declaring `pub mod layout;` and `pub mod confinement;` with a crate doc comment and
  no re-exports; each story adds its own `pub use` line when its types exist.
- Placeholders, each a doc comment naming the story that replaces it:
  `src/layout.rs` (replaced by `story:workspace-layout`), `src/confinement.rs` (replaced by
  `story:confinement-argv`), `src/main.rs` with an empty `fn main()` (replaced by
  `story:sandbox-cli`).
- `Taskfile.yml` with `check` (fmt --check, clippy `-D warnings`, test), `build` and `install`.
- `README.md`, `AGENTS.md`, `.gitignore`, `LICENSE` (Apache-2.0), `Cargo.lock`.

## Acceptance

`task check` exits 0 on the scaffold with the three placeholders in place.
