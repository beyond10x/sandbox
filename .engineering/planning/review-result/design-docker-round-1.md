---
format: aep.planning-md/1
id: review-result:design-docker-round-1
kind: review-result
status: active
title: Design critic, docker round 1
relations:
- reviews: epic:docker-backend
- reviews: story:docker-argv
- reviews: story:backend-selection
revision: 1
---
approve
needs-revision

story:docker-argv — its Acceptance (`cargo test --locked --test docker` passing) is `tests/docker.rs`, whose only test builds `Options { backend: Backend::Docker { .. }, .. }` and calls `Confinement::new(..).command()` — the `Backend` enum, `Options.backend` field and `Confinement`'s docker dispatch that story:backend-selection's own Behaviour section commits to adding — so this story's stated acceptance cannot pass until backend-selection has landed, the opposite of the recorded `story:backend-selection -> depends_on -> story:docker-argv` edge, and no edge records the order the test's own mechanics require — tests/docker.rs:9,45-50; .engineering/planning/story/backend-selection.md:31-36; .engineering/planning/story/docker-argv.md:41-49

**What I read:** 3 artifacts in full body (`epic:docker-backend`, `story:docker-argv`, `story:backend-selection` via `aep plan artifact show`), `aep plan artifact relations`, `aep plan artifact graph` (walked all edges it prints, including the 4 touching this set and the ones on `epic:sandbox-shell`'s implemented side, 27 total across the store), `aep plan artifact validate` (reports "valid" plus 3 unrelated missing-findings-block warnings on round-2 reviews of the earlier set — not a source of this finding), the prior `review-result:design-round-2.md` (to confirm the chain-serialisation point was already escalated once, per instruction, and is not repeated here), and the raw source files `src/lib.rs`, `src/docker.rs`, `src/confinement.rs`, `tests/docker.rs` to check what the acceptance test in `story:docker-argv`'s own scope actually exercises.

**What I could not establish:** whether the fix belongs in `story:docker-argv` (rewrite the test to call `docker::argv` directly without `Backend`/`Confinement`) or in an edge — a mutual-dependency edge would cycle with the existing `depends_on`, so this is a split abstraction, not just a missing edge, and I could not tell from the bodies which half the drafter intended to move. Whether the two stories' acceptances are independently checkable in isolation from this coupling is `plan-critic-acceptance`'s lane. Whether `epic:docker-backend`'s scope (e.g. `--allow-tiocsti` interaction, TTY handling) is fully covered by the two stories is `plan-critic-scope`'s lane. Whether the shared touches on `src/confinement.rs` and `src/lib.rs` between the two stories are safe to work concurrently is `plan-critic-parallel-safety`'s lane, and the depends_on edge already found there is not itself a finding of mine per this round's instruction.

```findings
- file: .engineering/planning/story/docker-argv.md
  line: 47
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "story:docker-argv's own acceptance runs tests/docker.rs, which exercises Options.backend: Backend::Docker and Confinement::new(..).command() -- the Backend enum and dispatch logic that story:backend-selection's Behaviour section commits to building -- so the acceptance cannot pass until backend-selection lands, the reverse of the recorded backend-selection -> depends_on -> docker-argv edge, and no edge records that order"
```
