---
format: aep.planning-md/1
id: review-result:acceptance-docker-round-2
kind: review-result
status: active
title: Acceptance critic, docker round 2
relations:
- reviews: epic:docker-backend
- reviews: story:docker-argv
- reviews: story:backend-selection
revision: 1
---
needs-revision

epic:docker-backend — "Done when" names only `task check` exits 0 on a machine with bwrap and Docker running, and this holds before the epic's work exists as much as after it — `task check` runs `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked` (Taskfile.yml:4-9), none of which name anything Docker-specific, so a repo with no `src/docker.rs`, no `Backend::Docker`, and no `--backend` flag exits 0 on this same check just as a repo with the full feature does; the clause names no docker-specific test or behavior the way both child stories' acceptances do — .engineering/planning/epic/docker-backend.md:52

What I read: all three drafted artifacts in full (`aep plan artifact show epic:docker-backend`, `aep plan artifact show story:docker-argv`, `aep plan artifact show story:backend-selection`), the round-1 record (`aep plan artifact show review-result:acceptance-docker-round-1`), `aep plan artifact kinds`, `aep plan artifact lifecycle epic`, `aep plan artifact lifecycle story`, the raw planning files under `.engineering/planning/{epic,story}/` for line numbers, `Taskfile.yml`, and the tree (`src/`, `tests/`, `git grep -n backend`) to check whether the named commands and tests exist. 3 of 3 ids read.

What I could not establish: whether `task check` currently passes end-to-end on a machine with a running Docker daemon (I confirmed the named unit test `argv_for_a_two_directory_layout_with_a_root_is_exact` exists in `src/docker.rs` and that `--backend`/`Backend::Docker` exist in `src/main.rs` and `src/confinement.rs`, but did not run the suite against a live daemon) — not needed for the acceptance-shape judgment above. Both round-1 findings are fixed as recorded: the epic's Done-when no longer joins two outcomes with "and," and story:docker-argv's acceptance now uses unscoped `cargo test --locked` rather than `--test docker`, so the unit test in `src/docker.rs` is exercised. I also noticed story:docker-argv's acceptance names only one of the two unit tests its own Tests section requires (the `caller_identity()` test at line 42 is unnamed in the Acceptance at line 51-53) — that reads as an ambition/thoroughness question rather than a shape defect (it is one observable statement with a real before/after transition), so it is not a finding of mine. I also noticed both stories' file-scope overlap with `epic:sandbox-shell`'s existing surface, but that is design/scope territory, not mine, and did not affect this verdict.

```findings
- file: .engineering/planning/epic/docker-backend.md
  line: 52
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the Done when clause names only `task check` exits 0 on a machine with bwrap and Docker running, which already holds today with no Docker backend, no src/docker.rs, and no --backend flag, since none of task check's steps (cargo fmt, cargo clippy, cargo test --locked) name anything docker-specific; the epic's done-state reads the same before the two stories are built as after"
```
