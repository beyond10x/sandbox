---
format: aep.planning-md/1
id: review-result:parallel-safety-docker-round-2
kind: review-result
status: active
title: Parallel-safety critic, docker round 2
relations:
- reviews: story:docker-argv
- reviews: story:backend-selection
revision: 1
---
approve

**What I read:** 2 artifacts in the drafted set — `story:docker-argv`, `story:backend-selection` — via `aep plan artifact show story:docker-argv`, `aep plan artifact show story:backend-selection`, `aep plan artifact show epic:docker-backend`, `aep plan artifact list --format json`, `aep plan artifact waves`, and `aep plan artifact graph`. Both scopes are **cited** by the store itself: `story:docker-argv` → `src/docker.rs`, `tests/docker.rs`; `story:backend-selection` → `src/confinement.rs`, `src/main.rs`, `src/lib.rs`, `tests/cli.rs`, `README.md`. Cited: 2. Inferred: 0. Unplaceable: 0.

The two scopes are disjoint — no shared file. `story:backend-selection` records `depends_on story:docker-argv` for the one place they connect (dispatch to `docker::argv`), and `aep plan artifact waves` places `story:docker-argv` in wave 1 and `story:backend-selection` in wave 2, reporting zero collision between them. The 6 collisions the CLI does report are all against `epic:sandbox-shell` stories (`story:confinement-argv`, `story:sandbox-cli`, `story:workspace-layout`), confirmed via `aep plan artifact list --format json` relations, and the task marks those historical, not concurrent.

I also checked the round-1 lane's own prior finding pattern: `review-result:design-docker-round-1` had flagged (now marked `fixed on story:docker-argv`) that the old `tests/docker.rs` test drove through `Confinement`/`Backend::Docker` — types `story:backend-selection` builds — creating a symbol-level ordering conflict against the recorded `depends_on` edge. The current `story:docker-argv` Tests section now states the integration test "calls `docker::argv` directly (not `Confinement`, which learns about Docker only in `story:backend-selection`)", which removes that coupling. That symbol-dependency direction is design's lane, not mine, but its resolution also removes what would otherwise have been my strongest candidate for an unnamed collision, so I record it as read.

**What I could not establish:** none.

**Out of my lane:** `review-result:design-docker-round-1`'s finding on `story:docker-argv`'s prior test/dependency ordering and `review-result:acceptance-docker-round-1`'s findings on the epic's "Done when" conjunction and the acceptance command's test-binary scoping are design/acceptance matters, already recorded there; I note only that the revision touching `story:docker-argv`'s Tests section does not introduce a new file-level collision with `story:backend-selection`.

```findings
[]
```
