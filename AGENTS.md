# AGENTS.md — sandbox

The contract for changing this repository. Org-wide rules live in `atlas/AGENTS.md`.

## What this repository owns

`b10x-sandbox`: a Rust library and CLI that respawns a process inside a sandbox with chosen host
directories mirrored under `/workspace`. It is a wrapper around `bwrap` (default) or `docker run`,
with no daemon, ledger, cgroup bounds or quotas. Substrate owns those; this repository copies
substrate's namespace and mount posture and nothing else.

## Serves

The objective of the collection this repository moves, by id from `atlas/ROADMAP.md` — the only
cross-repository roadmap, and the page that says what each id means and which evidence closes it:

- **O1 — governed reach.** The posture half of it: a confined workspace, host roots read-only, and
  a named refusal for every host path that is not `--dir` or `--ro`. Not the grant ledger, the
  cgroup bounds or the seccomp filter — those are substrate's, and this is a wrapper.

Derived from substrate: the bubblewrap isolation argument list in `src/confinement.rs` is a copy of
`crates/substrate-host/src/process.rs:62,1911-1937` (`USER_NAMESPACE_ARGV` plus the `command.args`
isolation set) at substrate 0.7.0, with the three differences named in that module's doc comment.
Admitted as a substrate-derived tool by Atlas ADR 0052 (2026-09-15).
Consumer: none yet — no repository beside this one names `b10x-sandbox`; a `grep -rlw` over the
sibling repositories' `*.md`, `*.toml`, `*.yaml`, `*.sh` and `*.rs` on 2026-09-15 matched only that
day's org-state review pages.

## Invariants

1. **Linux only; bubblewrap is the default backend and Docker the second.** A missing `bwrap` or
   `docker` is a named refusal, never a fallback to an unconfined process or to the other backend.
2. **Host directories enter only through `--dir` (writable, mirrored under `/workspace`) or `--ro`
   (read-only, at their own path).** Nothing else from the host is visible except `/usr`, `/bin`,
   `/lib`, `/lib64`.
3. **An input path is refused, never adjusted.** Not absolute, not canonical, not a directory,
   duplicated, nested in another input: each is a named error from `layout`.
4. **The isolation argument list in `src/confinement.rs` mirrors substrate's**
   (`crates/substrate-host/src/process.rs` at substrate 0.7.0), and `src/docker.rs` is its
   closest `docker run` equivalent with the differences listed in its module doc. Dropping an
   argument from either is a design change recorded in the planning store, not a cleanup.
5. **The planning store is `.engineering/planning/`, written only through `aep plan artifact`.**

## Gate

`task check`: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`,
`cargo test --locked`. Green here is the bar for `main`.

## Planning

Work is planned with AEP (`aep plan artifact list`). Read a story's body and `## Scope` before
touching a file; a story owns the files its scope names.
