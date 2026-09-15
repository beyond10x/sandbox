---
format: aep.planning-md/1
id: story:workspace-layout
kind: story
status: implemented
title: Compute the mirrored /workspace layout
relations:
- decomposes: epic:sandbox-shell
- depends_on: task:crate-scaffold
scope:
- confidence: cited
  path: src/layout.rs
- confidence: cited
  path: src/lib.rs
revision: 7
---
## Context

The whole tool rests on one pure computation: which host directory lands where under `/workspace`.
It is a library function so it can be tested without bubblewrap and reused by another program.
This story replaces the placeholder `src/layout.rs` from `task:crate-scaffold` and adds its
re-export line to `src/lib.rs`.

## Behaviour

`layout::Layout::plan(cwd, dirs)` returns a `Layout { ancestor, cwd, mappings }` with one
`Mapping { host, mount }` per input, the working directory first:

- no dirs: `cwd` maps to `/workspace` and is the ancestor;
- otherwise the longest common ancestor of `cwd` and every dir maps to `/workspace`, and each
  input maps to `/workspace/<input relative to ancestor>`; inputs under different top-level
  directories have `/` as ancestor and mirror from there;
- inputs are refused by name, never adjusted: `NotAbsolute`, `NotCanonical` (a symlink, `.` or
  `..` in any component), `NotADirectory`, `Duplicate`, and `Nested` (one input inside another,
  which is already writable through its parent — so an input can never equal the ancestor when
  there is more than one input);
- `layout::canonical_directory(path)` is public and performs the same validation for one path, so
  `story:confinement-argv` validates read-only roots with it rather than with a copy.

## Tests

Eight unit tests in `src/layout.rs`: the no-dir default; two siblings under one parent; inputs
under different top-level directories mirroring from `/`; and one test each for the `NotAbsolute`,
`NotCanonical` (both a `..` and a symlink), `NotADirectory` (both missing and a file), `Duplicate`
and `Nested` (both directions) refusals.

## Acceptance

`cargo test --locked --lib layout::` reports 8 passed and 0 failed.

## Scope

`src/layout.rs`, and one `pub use` line in `src/lib.rs`.
