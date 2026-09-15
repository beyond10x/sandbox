---
format: aep.planning-md/1
id: review-result:scope-round-2
kind: review-result
status: active
title: Scope critic, round 2
relations:
- reviews: task:crate-scaffold
- reviews: story:workspace-layout
- reviews: story:confinement-argv
- reviews: story:sandbox-cli
revision: 1
---
approve

**What I read:** 6 artifacts — the epic (`epic:sandbox-shell`, revision 2) and its four decomposed/implementing children (`task:crate-scaffold` rev 5, `story:workspace-layout` rev 7, `story:confinement-argv` rev 8, `story:sandbox-cli` rev 7) — each via `aep plan artifact show <id>`; the store graph via `aep plan artifact graph` (confirms no other artifact claims part of this epic, and shows the four round-1 review-results as the only other neighbors); `aep plan artifact kinds` and `aep plan artifact relations` this session; and `review-result:scope-round-1` for the prior round's findings.

Promises extracted from the revised epic (Outcome bullets, the behavioral Boundary lines, and Done-when): 12 — (1) no-`--dir` default writable-at-`/workspace`, (2) `--dir` common-ancestor mapping mechanics with read-only synthesized parents and pwd-relative chdir, (3) mirror-from-`/` behavior for inputs under different top-level directories, (4) no-command defaults to `$SHELL`/`/bin/sh`, (5) trailing `-- CMD...` replaces the command, (6) `--ro` read-only bind at own absolute path, never writable, (7) fixed substrate namespace/mount posture including conditional `--net`, (8) `--dry-run` prints argv, (9) library exposes `Layout`/`Confinement`/`Options` including an `env` field the CLI does not expose, (10) Linux/bubblewrap-only with a named refusal on missing `bwrap`, (11) TIOCSTI-sysctl check refusing unless `0` or `--allow-tiocsti` given, (12) `task check` green as the aggregate completion bar. All 12 traced to at least one drafted item (7 to `story:confinement-argv`, mapping/mirroring to `story:workspace-layout`, command handling and `--dry-run`/`--ro`/`--net`/`--allow-tiocsti` surfacing to `story:sandbox-cli`, the aggregate bar to `task:crate-scaffold`'s `Taskfile.yml` `check` target plus the full decomposition). No gap found.

All three round-1 findings (the `network` option, the `--ro`/`read_only_roots` option, and the trailing `-- CMD...` argument) are now promised in the epic's own body (revision 2's Outcome and the `--net` clause), so what was reach-beyond-parent in round 1 is now traceable. Checked the epic's exclusions (no daemon/ledger/cgroup/quotas, no seccomp filter this release, no entity persisted/no ESS, no `--new-session`) against all four items — none is claimed by any drafted item.

**What I could not establish:** whether `--remount-ro /workspace` in `story:confinement-argv`'s argv sequence leaves the bound mappings themselves writable as the epic requires, or whether it also re-locks them — that is a correctness/design question, not a scope-coverage one, so it is out of my lane for `plan-critic-design`/`plan-critic-acceptance`, not scored here.

```findings
[]
```
