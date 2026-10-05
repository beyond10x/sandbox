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
Admitted as a substrate-derived tool by Atlas ADR 0052 (accepted 2026-09-15). That ADR deferred both
the `sandbox` catalog row and the `sandbox-derived-from-substrate-20260915` lineage record until
`beyond10x/sandbox` existed as a GitHub repository. **It exists since 2026-09-15**, public, and this
tree is published to it, so the condition the ADR waited on is met and the two catalog records are
owed. Until Atlas writes them this repository has an admission and a remote but no catalog identity.
Consumer: **`atlas`, since 2026-09-18.** `atlas/scripts/o6-loop.sh:74,93` requires this binary by
name and refuses to measure without it, and `atlas/crates/o6-loop` records the `bwrap` argv it
produced into every observation it writes (`confinement.argv_digest`). The O6 self-improvement loop
builds and replays each candidate inside this sandbox with the network namespace unshared, which is
what lets it say "no provider was called" as a property of the run rather than a claim.

The sentence this replaces read "Consumer: none yet — no repository beside this one names
`b10x-sandbox`", from a `grep -rlw` over the sibling repositories on 2026-09-15 that matched only
that day's org-state review pages. It was true when written and is now false; it is recorded here
rather than deleted, because a claim that changed is worth more than a claim that was quietly
corrected.

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

`task check` is the bar for `main`. Its steps, each runnable alone:

| Step | Command |
|---|---|
| format | `cargo fmt --all --check` |
| lints | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| tests | `cargo test --workspace --locked` |
| generated docs current | `task docs-check` (`cargo run --locked -p sandbox-docs -- generate --check`) |

The toolchain is Rust 1.97, pinned by `rust-toolchain.toml`. `tests/bubblewrap.rs` and
`tests/docker.rs` need `bwrap` and a Docker daemon; without one they print
`<program> absent: confinement not observed` and pass without observing anything. CI (`Gate`,
`.github/workflows/gate.yml`) installs bubblewrap and fails a run in which either case went
absent, so a local run without them is not evidence for those two files.

Build into a target directory shared by every worktree of this repository
(`CARGO_TARGET_DIR=~/.cache/b10x-target/sandbox`), with `CARGO_INCREMENTAL=0`.

## Documentation

The site is `website/` (Docusaurus with `@beyond10x/docs-system`), served at `/sandbox/` by its
own Pages deployment. README.md is for people opening the repository; this file is for agents; the
site is for users. A change to a command, an option, a refusal or the argument list updates all
three in the same commit, and `CHANGELOG.md` under **Unreleased**.

| File | Written by |
|---|---|
| `website/docs/reference/cli.md` | `sandbox-docs`, from `src/cli.rs` (`b10x_sandbox::cli::Cli`) |
| `website/docs/reference/argv.md` | `sandbox-docs`, by calling the library for fixed layouts |
| `website/docs/reference/crates.md` | `sandbox-docs`, from `cargo metadata --no-deps` |
| `website/data/status.json` | `sandbox-docs`, from `crates/sandbox-docs/src/status.rs`; a shipped item names its test as `file::function`, and generation fails when that test is gone |
| everything else under `website/` | hand; every command on a page is run before it is pasted |

- `task docs-generate` rewrites the generated files; never edit them by hand.
- `task website` builds the site (`npm ci && npm run build` in `website/`), as the
  `Documentation validation` workflow (`.github/workflows/pages.yml`) does. On a bot push to
  `main`, `Documentation site` (`.github/workflows/b10x-docs-site.yml`) deploys that exact build.
- `b10x.docs.yaml` and `.github/workflows/b10x-docs-pages.yml` are the retired unified site's,
  generated by Atlas. They stay until Atlas lists this repository as independently documented;
  deleting them first is refused organisation-wide. Do not edit them.
- No absolute home-directory path on any page or in any test fixture added from now on: write `~`
  or a neutral path such as `/srv/work`.

## Planning

Work is planned with AEP (`aep plan artifact list`). Read a story's body and `## Scope` before
touching a file; a story owns the files its scope names.

## Never

- Drop or reorder an argument of either backend list as a cleanup (invariant 4).
- Add a fallback when a backend program is missing (invariant 1).
- Edit a generated documentation file, or claim a capability as shipped without the test that
  holds it.
