---
format: aep.planning-md/1
id: review-result:acceptance-docker-round-1
kind: review-result
status: active
title: Acceptance critic, docker round 1
relations:
- reviews: epic:docker-backend
- reviews: story:docker-argv
- reviews: story:backend-selection
revision: 1
---
approve
needs-revision

epic:docker-backend — "Done when" joins two independently-checkable outcomes with "and" ("both stories are implemented" and "`task check` is green"), so one can hold while the other fails and the epic's done-state stays undecided — .engineering/planning/epic/docker-backend.md:52

story:docker-argv — the acceptance command `cargo test --locked --test docker` scopes to the integration-test binary only (`cargo test --help`: "--test [<NAME>] Test only the specified test target"), so it never runs the unit test in `src/docker.rs` the story's own Tests section requires, leaving that test's presence or correctness unchecked by the acceptance — .engineering/planning/story/docker-argv.md:49 (unit test named at :41)

What I read: all three drafted artifacts in full — `aep plan artifact show epic:docker-backend`, `aep plan artifact show story:docker-argv`, `aep plan artifact show story:backend-selection` — plus `aep plan artifact kinds`, `aep plan artifact lifecycle story`, `cargo test --help`, and the raw files under `.engineering/planning/{epic,story}/` to get line numbers. 3 of 3 ids read.

What I could not establish: whether `task check` and `cargo test --locked --test docker` actually pass today, since `src/docker.rs`, `tests/docker.rs` and the `--backend` CLI flag don't exist yet in this pre-implementation draft — not evaluable and not needed for an acceptance-shape judgment. I also noticed the epic's "Boundaries" and the two stories' file-scope overlap (both touch files under `epic:sandbox-shell`'s existing surface) but that is design/scope territory, not mine, and did not affect this verdict.

```findings
- file: .engineering/planning/epic/docker-backend.md
  line: 52
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the Done when clause joins two independently-checkable outcomes with \"and\" (\"both stories are implemented\" and \"`task check` is green\"), so one can hold while the other fails and the epic's done-state stays undecided"
- file: .engineering/planning/story/docker-argv.md
  line: 49
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance command `cargo test --locked --test docker` scopes to the integration-test binary only, so it never runs the unit test in src/docker.rs that the story's Tests section (line 41) requires, leaving that test's presence or correctness unchecked by the acceptance"
```
