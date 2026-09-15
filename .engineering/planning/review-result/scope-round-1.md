---
format: aep.planning-md/1
id: review-result:scope-round-1
kind: review-result
status: active
title: Scope critic, round 1
relations:
- reviews: task:crate-scaffold
- reviews: story:workspace-layout
- reviews: story:confinement-argv
- reviews: story:sandbox-cli
revision: 1
---
approve

Wait — findings below require `needs-revision`. Correcting:

needs-revision

story:confinement-argv — claims an optional `network: bool` that skips `--unshare-net` (also surfaced as sandbox-cli's `--net`), but epic:sandbox-shell's Outcome fixes the posture as unconditionally "ipc/pid/net/uts unshared" and names no exception for network — .engineering/planning/story/confinement-argv.md:24,29
story:confinement-argv — claims a `read_only_roots` option that read-only-binds arbitrary caller-named host directories at their own absolute path (surfaced as sandbox-cli's `--ro`, e.g. `~/.cargo`), which epic:sandbox-shell's Outcome does not name — it promises only `--dir` (writable, mirrored under `/workspace`) and the fixed substrate system read-only mounts — .engineering/planning/story/confinement-argv.md:24-25,32
story:sandbox-cli — claims a `-- <CMD>...` argument that replaces the sandboxed shell with an arbitrary caller-supplied command, but epic:sandbox-shell's Outcome promises only that the tool "replaces the current process with the caller's shell" — .engineering/planning/story/sandbox-cli.md:25,32

What I read: the epic (`epic:sandbox-shell`) and all four drafted children (`task:crate-scaffold`, `story:workspace-layout`, `story:confinement-argv`, `story:sandbox-cli`), each via `aep plan artifact show <id>`; the store graph via `aep plan artifact graph` (confirms no other artifact claims part of this epic); `aep plan artifact kinds` and `aep plan artifact relations`; and `cat -n` on the five underlying `.md` files for line citations.

Promises extracted from the epic (Outcome + the behavior-bearing Boundary lines + Done-when): 8 — (1) exec-replace into the caller's shell, (2) no-`--dir` default mapping, (3) `--dir` common-ancestor mapping with read-only tmpfs parents and pwd-relative chdir, (4) fixed substrate namespace/mount posture, (5) library-reusable computation, (6) named refusal on missing `bwrap`, (7) TIOCSTI-sysctl check with overridable refusal, (8) `task check` green as the aggregate completion bar. All 8 traced to at least one drafted item; no gaps found.

What I could not establish: whether the `Options.env` custom-injection field in `story:confinement-argv` (never exposed by any flag in `story:sandbox-cli`'s usage line) is an intentional library-only surface or a dropped CLI feature — that is an internal-consistency question for `plan-critic-design`, not scope, so I did not score it; I also did not verify the cited `crates/substrate-host/src/process.rs:1911-1937` argv against confinement-argv's actual bwrap-argument list — that correctness check belongs to `plan-critic-acceptance`/`plan-critic-design`, not this lane.

```findings
- file: .engineering/planning/story/confinement-argv.md
  line: 29
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: story:confinement-argv claims an optional `network` bool that skips `--unshare-net` (also surfaced as sandbox-cli's `--net`), but epic:sandbox-shell's Outcome fixes the posture as unconditionally "ipc/pid/net/uts unshared" and names no exception for network
- file: .engineering/planning/story/confinement-argv.md
  line: 32
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: story:confinement-argv claims a `read_only_roots` option that read-only-binds arbitrary caller-named host directories at their own absolute path (surfaced as sandbox-cli's `--ro`), which epic:sandbox-shell's Outcome does not name; it promises only `--dir` (writable, mirrored under /workspace) and the fixed substrate system read-only mounts
- file: .engineering/planning/story/sandbox-cli.md
  line: 32
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: story:sandbox-cli claims a `-- <CMD>...` argument that replaces the sandboxed shell with an arbitrary caller-supplied command, but epic:sandbox-shell's Outcome promises only that the tool "replaces the current process with the caller's shell"
```
