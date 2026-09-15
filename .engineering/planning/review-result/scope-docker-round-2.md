---
format: aep.planning-md/1
id: review-result:scope-docker-round-2
kind: review-result
status: active
title: Scope critic, docker round 2
relations:
- reviews: story:docker-argv
- reviews: story:backend-selection
revision: 1
---
approve

**What I read:** 3 artifacts in full body via `aep plan artifact show` — `epic:docker-backend`, `story:docker-argv`, `story:backend-selection` — plus `aep plan artifact graph`, `aep plan artifact kinds`, `aep plan artifact relations` (confirming `decomposes` is the drafted-from edge and that no other artifact decomposes this epic), `aep plan artifact history` on both stories (to see what round-1 acceptance/design findings changed since round 1's scope review, since this is round 2), and `review-result:scope-docker-round-1`, `review-result:design-docker-round-1`, `review-result:acceptance-docker-round-1` for prior-round context.

**Coverage check:** I independently extracted 7 outcome promises from `epic:docker-backend`'s Outcome section (`.engineering/planning/epic/docker-backend.md`): (1) `--backend docker` container run with the mirrored `Layout` from `--image`; (2) `--backend bubblewrap` stays default, unchanged; (3) the full Docker posture flag list; (4) same env baseline via `-e`, image vars otherwise untouched; (5) default command `/bin/sh`, trailing `-- CMD...` override; (6) missing `docker` binary is a named refusal; (7) `Backend::{Bubblewrap, Docker { image, tty }}` on `Options` and `Confinement::program()`. All 7 traced: (1) split cleanly between `story:docker-argv`'s `argv()` bind/`-w` construction and `story:backend-selection`'s `--image` CLI flag; (3), (4) land whole in `story:docker-argv`'s numbered Behaviour list; (2), (5)'s CLI half, (6), (7) land in `story:backend-selection`'s Behaviour section. 7/7 promises traced, each to exactly one item or one clean split, none doubled, none narrowed.

The history shows `story:docker-argv` changed twice since drafting (revision 4 for `review-result:acceptance-docker-round-1`, revision 5 for `review-result:design-docker-round-1` — the latter is why the current Tests section now says the integration test "calls `docker::argv` directly (not `Confinement`, which learns about Docker only in `story:backend-selection`)"), and the epic's own "Done when" was simplified from a two-clause "and" to the current single clause. Neither fix touched scope: both stories still cover the same 7 promises with the same split, and the Boundaries (no daemon management, no image build/pull, TIOCSTI restricted to bubblewrap — restated verbatim in `story:backend-selection`'s Behaviour, "the TIOCSTI check only for bubblewrap" — and no entity introduced) hold unchanged. Nothing in either story's Behaviour/Tests/Scope reaches past a sentence in the epic.

**What I could not establish:** none — every promise had a direct match in one of the two story bodies, no revision since round 1 touched coverage, and the graph confirms these are the only two items decomposing this epic. (The design and acceptance defects fixed between round 1 and now are `plan-critic-design`'s and `plan-critic-acceptance`'s lanes, not restated here as they carry no scope consequence.)

```findings
[]
```
