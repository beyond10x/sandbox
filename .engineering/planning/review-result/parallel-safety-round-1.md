---
format: aep.planning-md/1
id: review-result:parallel-safety-round-1
kind: review-result
status: active
title: Parallel-safety critic, round 1
relations:
- reviews: task:crate-scaffold
- reviews: story:workspace-layout
- reviews: story:confinement-argv
- reviews: story:sandbox-cli
revision: 1
---
needs-revision
task:crate-scaffold — both this and story:sandbox-cli land on `src/main.rs` (cited: task's Work creates it, story's scope owns it exclusively) and neither body names the other, contradicting the task's own stated rule two lines above that "no story edits a file another story owns" — .engineering/planning/task/crate-scaffold.md:24

What I read: 4 artifacts (`task:crate-scaffold`, `story:workspace-layout`, `story:confinement-argv`, `story:sandbox-cli`) via `aep plan artifact show <id>` (all four), `aep plan artifact list --format json`, `aep plan artifact waves`, and `aep plan artifact graph`.

Surface placement: 4 cited (task:crate-scaffold from its Work section, `.engineering/planning/task/crate-scaffold.md:19-26`; the three stories from their declared `scope:` frontmatter, `confidence: cited` in each), 0 inferred, 0 unplaceable.

What I could not establish: none — every item names a concrete file list, either in `scope:` frontmatter or in prose (`Work`). Note `aep plan artifact waves` reports "0 collision(s), 0 unassessed" but never places `task:crate-scaffold` into any wave and it carries no `scope:` field, so that automated check never compared it against `story:sandbox-cli`'s scope — the `src/main.rs` overlap above is invisible to it, not contradicted by it. Separately, whether `task:crate-scaffold`'s `src/lib.rs` bullet (declaring `pub mod layout;`/`pub mod confinement;` and re-exporting types that don't exist until later stories land, plus an acceptance of `task check` passing on the "empty scaffold") is internally consistent is a design/acceptance question, out of my lane — flagging it here only so it isn't lost, not as a concurrency finding.

(Recorded by the orchestrator with the `message` value below quoted, because the store's YAML reader refused the unquoted form; the words are the critic's.)

```findings
- file: .engineering/planning/task/crate-scaffold.md
  line: 24
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "both task:crate-scaffold and story:sandbox-cli land on `src/main.rs` (cited: task's Work creates it, story's scope owns it exclusively) and neither body names the other, contradicting the task's own stated rule two lines above that \"no story edits a file another story owns\""
```
