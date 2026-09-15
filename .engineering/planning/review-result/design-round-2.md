---
format: aep.planning-md/1
id: review-result:design-round-2
kind: review-result
status: active
title: Design critic, round 2
relations:
- reviews: task:crate-scaffold
- reviews: story:workspace-layout
- reviews: story:confinement-argv
- reviews: story:sandbox-cli
revision: 1
---
needs-revision

story:sandbox-cli — the `depends_on` chain (task:crate-scaffold → story:workspace-layout → story:confinement-argv → story:sandbox-cli) still has no branching, so the set can only be worked in that single order and the split into four items still buys nothing over one — this is the same warning `review-result:design-round-1` raised, recorded there as "escalated" rather than fixed, and the graph is unchanged since — `aep plan artifact graph`

**What I read:** 5 artifacts in full body (`aep plan artifact show` on `epic:sandbox-shell`, `task:crate-scaffold`, `story:workspace-layout`, `story:confinement-argv`, `story:sandbox-cli`), plus `aep plan artifact relations`, `aep plan artifact graph`, `aep plan artifact validate` (reports "valid" — not a source of either finding), `review-result:design-round-1` in full, and the raw files `.engineering/planning/task/crate-scaffold.md`, `story/workspace-layout.md`, `story/confinement-argv.md` for the placeholder/pub-use wording. I walked all 23 edges the graph prints (16 `reviews`, 3 `decomposes`, 3 `depends_on`, 1 `implements`), which includes edges to `epic:sandbox-shell` and the four `review-result` nodes — outside the four-id set I was handed — and found no cycle: `decomposes`/`implements` point into the epic without returning, `depends_on` forms one simple acyclic chain, and `reviews` points from the review-result nodes into the four items without a return edge.

Round 1's blocker is fixed: `task:crate-scaffold` now explicitly creates the three placeholder files (`src/layout.rs`, `src/confinement.rs`, `src/main.rs`) and scopes its own acceptance to "the scaffold with the three placeholders in place," so `task check` no longer depends on files the graph places later — `.engineering/planning/task/crate-scaffold.md:14-16,25-30,36`. Round 1's open question about `layout::canonical_directory` is also resolved: `story:workspace-layout` now explicitly commits to exporting it, and `story:confinement-argv` cites it by name, so the earlier "loose cross-reference" is now a recorded, exact dependency — `.engineering/planning/story/workspace-layout.md:31-33`, `.engineering/planning/story/confinement-argv.md:29`.

**What I could not establish:** nothing outstanding — the one open item from round 1 (the `canonical_directory` cross-reference) is now resolved by the text cited above. Whether the four items' acceptance criteria are independently checkable is `plan-critic-acceptance`'s lane. Whether the set covers everything `epic:sandbox-shell` describes (e.g. `--allow-tiocsti`, `env` on the library only) is `plan-critic-scope`'s lane. Whether the shared touches on `src/lib.rs` across `task:crate-scaffold`, `story:workspace-layout`, and `story:confinement-argv` are safe given the `depends_on` chain is `plan-critic-parallel-safety`'s lane — I note only that the chain is what makes the question moot in practice, which is shape, not safety.

```findings
- file: aep plan artifact graph
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the depends_on chain (task:crate-scaffold -> story:workspace-layout -> story:confinement-argv -> story:sandbox-cli) still has no branching, so the set can only be worked in that single order and the split into four items still buys nothing over one; review-result:design-round-1 raised this and recorded it as escalated rather than fixed, and the graph is unchanged"
```
