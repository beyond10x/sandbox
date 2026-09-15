---
format: aep.planning-md/1
id: review-result:design-docker-round-2
kind: review-result
status: active
title: Design critic, docker round 2
relations:
- reviews: story:docker-argv
- reviews: story:backend-selection
revision: 1
---
approve

**What I read:** 2 artifacts in full body (`story:docker-argv`, `story:backend-selection`, via `aep plan artifact show`), `aep plan artifact relations` (14 edge meanings), `aep plan artifact graph` (53 edges walked across the whole store, not just the two ids handed to me — including the `epic:sandbox-shell` chain and every `review-result` edge), `aep plan artifact validate` (reports `valid`, plus 5 pre-existing missing-findings-block warnings on unrelated round-1/round-2 reviews, none touching this set), `review-result:design-docker-round-1` (the prior finding and its outcome), and the source files `tests/docker.rs`, `src/docker.rs`, `src/confinement.rs` to confirm the fix actually landed in code, not just in the story body.

The round-1 defect — `tests/docker.rs` exercising `Options.backend`/`Confinement`, which made `story:docker-argv`'s acceptance depend on `story:backend-selection` landing first, backwards from the recorded `depends_on` edge — is resolved: `tests/docker.rs:10,50-57` now imports and calls `b10x_sandbox::docker::{argv, caller_identity}` directly, and `story:docker-argv`'s own Tests section states this explicitly ("calls `docker::argv` directly (not `Confinement`, which learns about Docker only in `story:backend-selection`)"). The single `story:backend-selection --depends_on--> story:docker-argv` edge now points the direction the code actually requires (`src/confinement.rs:163-164` calls `docker::caller_identity()` and `docker::argv(...)`), and is a two-item dependency, not the multi-item serialising chain already escalated in `review-result:design-round-2` on the earlier set — that escalation is not repeated here. The two stories' scopes (`src/docker.rs`+`tests/docker.rs` vs. `src/confinement.rs`,`src/main.rs`,`src/lib.rs`,`tests/cli.rs`,`README.md`) are disjoint, and neither body's Behaviour section reaches into the other's internals beyond the one function signature the dependency edge already declares. No cycle: following the two substantive edges out of this set (`decomposes` x2, `depends_on` x1) plus the epic-level `informed_by` reaches no artifact a second time.

**What I could not establish:** `story:docker-argv`'s Behaviour section documents `docker::argv(layout, options, image, tty, command)` (5 parameters) while the implemented function and its call site both take 6 (`identity` inserted before `command` — `src/docker.rs:40-47`, `tests/docker.rs:50-57`, `src/confinement.rs:164`). This is a body-vs-code accuracy question, not a decomposition-shape defect — I could not tell whether it belongs to `plan-critic-acceptance`'s lane (does the acceptance text still describe a checkable interface) or is simply stale prose on an already-implemented story, so I raise it here without a verdict-affecting finding rather than guess which lane owns it.

```findings
[]
```
