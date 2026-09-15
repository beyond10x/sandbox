---
format: aep.planning-md/1
id: review-result:design-round-1
kind: review-result
status: active
title: Design critic, round 1
relations:
- reviews: task:crate-scaffold
- reviews: story:workspace-layout
- reviews: story:confinement-argv
- reviews: story:sandbox-cli
revision: 1
---
needs-revision

task:crate-scaffold — its Work section has `src/lib.rs` declare `pub mod layout;` and `pub mod confinement;`, but `src/layout.rs` and `src/confinement.rs` are the deliverables of story:workspace-layout and story:confinement-argv, both of which `depends_on` this task and so are built after it — meaning this task's own acceptance ("`task check` exits 0 on the empty scaffold") cannot pass as scoped, since the crate will not compile without files the graph places later in the queue — .engineering/planning/task/crate-scaffold.md:23

story:sandbox-cli — the four `depends_on` edges form one straight line (task:crate-scaffold → story:workspace-layout → story:confinement-argv → story:sandbox-cli) with no branching, so the set can only be worked in that single order and the split into four items buys nothing over one — `aep plan artifact graph`

What I read: 5 artifacts in full body (`aep plan artifact show` on epic:sandbox-shell, task:crate-scaffold, story:workspace-layout, story:confinement-argv, story:sandbox-cli), plus `aep plan artifact relations`, `aep plan artifact graph`, `aep plan artifact validate` (reports "valid" — neither finding above is something it caught), and the raw file `.engineering/planning/task/crate-scaffold.md` for line numbers. I walked all 7 edges the graph prints, which includes edges to epic:sandbox-shell — outside the four-id set I was handed — and found no cycle: the three `decomposes` edges and the one `implements` edge all point into the epic without returning, and the three `depends_on` edges form a simple acyclic chain, not a loop.

What I could not establish: story:confinement-argv's body says read-only-root inputs get "validation as in the layout story," but story:workspace-layout's own Behaviour and Scope describe that validation only as internal to `Layout::plan`'s handling of `--dir` paths, not as a function it commits to exporting for reuse on a different path list — I could not tell whether this is a real hidden dependency or just loose cross-reference wording, so I did not turn it into a finding. Whether each story's acceptance criteria are independently checkable is plan-critic-acceptance's lane, not mine. Whether the four items fully cover the epic's stated boundaries (e.g. the TIOCSTI sysctl check, the `--dry-run` path) is plan-critic-scope's lane. Whether the non-overlapping file scopes are safe to run in parallel despite the depends_on chain is plan-critic-parallel-safety's lane — I note only that the chain's existence is what makes their answer moot in practice, which is a comment on shape, not on safety.

```findings
- file: .engineering/planning/task/crate-scaffold.md
  line: 23
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: its Work section has src/lib.rs declare `pub mod layout;` and `pub mod confinement;`, but src/layout.rs and src/confinement.rs are the deliverables of story:workspace-layout and story:confinement-argv, both of which depends_on this task and so are built after it — meaning this task's own acceptance ("task check exits 0 on the empty scaffold") cannot pass as scoped, since the crate will not compile without files the graph places later in the queue
- file: aep plan artifact graph
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the four depends_on edges form one straight line (task:crate-scaffold → story:workspace-layout → story:confinement-argv → story:sandbox-cli) with no branching, so the set can only be worked in that single order and the split into four items buys nothing over one
```
