---
format: aep.planning-md/1
id: review-result:parallel-safety-round-2
kind: review-result
status: active
title: Parallel-safety critic, round 2
relations:
- reviews: task:crate-scaffold
- reviews: story:workspace-layout
- reviews: story:confinement-argv
- reviews: story:sandbox-cli
revision: 1
---
approve

What I read: 4 artifacts (`task:crate-scaffold`, `story:workspace-layout`, `story:confinement-argv`, `story:sandbox-cli`) via `aep plan artifact show <id>` (all four), `aep plan artifact list --format json`, `aep plan artifact waves`, and `aep plan artifact graph`; cross-checked the actual `src/` and `tests/` tree (`find`, `cat src/lib.rs src/confinement.rs src/main.rs`) against the declared scopes.

Surface placement: 4 cited, 0 inferred, 0 unplaceable — `task:crate-scaffold` from its `Work` section (`.engineering/planning/task/crate-scaffold.md:19-32`), the three stories from their `scope:` frontmatter (`confidence: cited` in each).

Round-1's finding (`review-result:parallel-safety-round-1`) was that `task:crate-scaffold` and `story:sandbox-cli` both land on `src/main.rs` while the task's own text claimed "no story edits a file another story owns." That contradiction is gone: the task's Context now states outright "Three source files are created here as placeholders and then **replaced** by the story named beside each, which is the one deliberate overlap in this set," and the `Work` bullet names the story next to each of the three placeholder files (`src/layout.rs` → `story:workspace-layout`, `src/confinement.rs` → `story:confinement-argv`, `src/main.rs` → `story:sandbox-cli`) — `.engineering/planning/task/crate-scaffold.md:13-17,27-30`.

The remaining overlap the CLI itself flags (`aep plan artifact waves`: `collision: story:confinement-argv story:workspace-layout src/lib.rs`) is named on the plan side too: the task's Context calls `src/lib.rs` one of "the files they share ... created once here," each story's body says it "adds its own re-export line to src/lib.rs," and `story:confinement-argv` `depends_on story:workspace-layout` so the two never land in the same wave (`aep plan artifact waves`: `wave 1 story:workspace-layout`, `wave 2 story:confinement-argv`). Per the rubric this is a validator-reported item that the plan already accounts for, not a fresh finding.

No other pair shares a file: `story:workspace-layout` (`src/layout.rs`, `src/lib.rs`) and `story:sandbox-cli` (`src/main.rs`, `tests/cli.rs`) don't intersect; `story:confinement-argv` (`src/confinement.rs`, `src/lib.rs`, `tests/bubblewrap.rs`) and `story:sandbox-cli` don't intersect — `sandbox-cli` only imports the library's public types, it doesn't edit `lib.rs`.

What I could not establish: none — every item names a concrete file set, either in `scope:` frontmatter or in prose (`Work`).

Out of my lane, not affecting this verdict: the store's `status` fields (`story:confinement-argv` = active, `story:sandbox-cli` = draft) disagree with the working tree, where `src/confinement.rs` and a fully-implemented `src/main.rs` (well past an "empty `fn main()`" placeholder) already exist — a scope/acceptance-lifecycle question, not a concurrency one.

```findings
[]
```
