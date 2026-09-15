---
format: aep.planning-md/1
id: review-result:scope-docker-round-1
kind: review-result
status: active
title: Scope critic, docker round 1
relations:
- reviews: story:docker-argv
- reviews: story:backend-selection
revision: 1
---
approve

**What I read:** 3 artifacts — `epic:docker-backend`, `story:docker-argv`, `story:backend-selection` — via `aep plan artifact show <id>` for each, plus `aep plan artifact graph`, `aep plan artifact kinds`, and `aep plan artifact relations` to confirm `decomposes` is the drafted-from edge and that no other artifact already claims part of this epic.

**Coverage check:** I extracted 7 outcome promises from `epic:docker-backend`'s Outcome section (`.engineering/planning/epic/docker-backend.md`):
1. `--backend docker` container run with the mirrored `Layout` (writable/read-only binds, `-w`)
2. `--backend bubblewrap` stays default, unchanged behaviour
3. the full Docker posture flag list (`--rm`, `--user`, `--network none` unless `--net`, `--cap-drop ALL`, `--security-opt no-new-privileges`, `--read-only`, `--tmpfs /tmp`, tmpfs `/workspace` 0555, `-i`/`-t`)
4. same env baseline via `-e`, image vars otherwise untouched
5. default command `/bin/sh`, trailing `-- CMD...` override
6. missing `docker` binary is a named refusal
7. `Backend::{Bubblewrap, Docker { image, tty }}` on `Options` and `Confinement::program()`

All 7 traced: 1, 3, 4, 5 (arg-building half), 6 (partial) land in `story:docker-argv`'s `argv()` behaviour spec; 2, 5 (default-command/CLI half), 6 (`ProgramMissing`), 7 land in `story:backend-selection`'s Behaviour section. 7/7 promises traced to exactly one claiming item, none doubled, none narrowed. The Boundaries (no daemon management, no image build/pull, TIOCSTI only for bubblewrap, no entity introduced) are respected — the TIOCSTI restriction is explicitly restated in `story:backend-selection`, and neither story adds daemon, build, or entity-persistence work. Nothing in either story's Behaviour/Tests/Scope reaches past a sentence in the epic.

**What I could not establish:** none — every promise had a direct match in one of the two story bodies, and the graph confirms these are the only two items decomposing this epic.

```findings
[]
```
