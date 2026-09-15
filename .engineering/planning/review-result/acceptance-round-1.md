---
format: aep.planning-md/1
id: review-result:acceptance-round-1
kind: review-result
status: active
title: Acceptance critic, round 1
relations:
- reviews: task:crate-scaffold
- reviews: story:workspace-layout
- reviews: story:confinement-argv
- reviews: story:sandbox-cli
revision: 1
---
needs-revision

story:workspace-layout — the acceptance is one sentence joining five independent outcomes with "and" (the five refusals, the no-dir default, a two-sibling layout, an ancestor-equals-input layout, and `cargo test` exiting 0), so any one of the named test scenarios can be missing or fail while `cargo test` still exits 0, and the story is neither done nor not done — .engineering/planning/story/workspace-layout.md:37-38

story:confinement-argv — the acceptance bundles three independent checks across two semicolon-joined sentences (the unit-test argv equality, the present-`bwrap` integration test's three observations, and the absent-`bwrap` fallback message), so one can pass while another fails with no single answer for "done" — .engineering/planning/story/confinement-argv.md:46-50

story:sandbox-cli — the acceptance names two independent integration tests, explicitly "Both" (the dry-run success case and the non-directory failure case), in one semicolon-joined sentence, so one test can pass while the other fails and the story is neither done nor not done — .engineering/planning/story/sandbox-cli.md:39-42

What I read: all 4 requested artifacts in full via `aep plan artifact show` (task:crate-scaffold, story:workspace-layout, story:confinement-argv, story:sandbox-cli), plus `aep plan artifact kinds` and `aep plan artifact lifecycle task|story`, and the raw store files under `.engineering/planning/{task,story}/*.md` to get exact line citations.

What I could not establish: none — every acceptance section is present and its wording is legible on its own; no artifact required tree inspection to judge (all four are draft, pre-implementation, so thinness of the Behaviour/Scope sections is not mine to flag).

```findings
- file: .engineering/planning/story/workspace-layout.md
  line: 37
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: the acceptance is one sentence joining five independent outcomes with "and" (the five refusals, the no-dir default, a two-sibling layout, an ancestor-equals-input layout, and `cargo test` exiting 0), so any one of the named test scenarios can be missing or fail while `cargo test` still exits 0, and the story is neither done nor not done
- file: .engineering/planning/story/confinement-argv.md
  line: 46
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: the acceptance bundles three independent checks across two semicolon-joined sentences (the unit-test argv equality, the present-`bwrap` integration test's three observations, and the absent-`bwrap` fallback message), so one can pass while another fails with no single answer for "done"
- file: .engineering/planning/story/sandbox-cli.md
  line: 39
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: the acceptance names two independent integration tests, explicitly "Both" (the dry-run success case and the non-directory failure case), in one semicolon-joined sentence, so one test can pass while the other fails and the story is neither done nor not done
```
