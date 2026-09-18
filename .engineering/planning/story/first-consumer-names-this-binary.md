---
format: aep.planning-md/1
id: story:first-consumer-names-this-binary
kind: story
status: draft
title: A repository outside this one names b10x-sandbox
summary: atlas/scripts/o6-loop.sh requires this binary by name and refuses to measure without it; the AGENTS.md claim that nothing else names it is retired by a fact rather than a decision.
revision: 1
---
## Outcome
`b10x-sandbox` has a consumer outside this repository, and `AGENTS.md` says so.

## What consumes it
`atlas/scripts/o6-loop.sh:74,93` requires this binary by name and refuses to measure without it. `atlas/crates/o6-loop` records the `bwrap` argv it produced into every observation it writes, as `confinement.argv_digest`. The O6 self-improvement loop builds and replays each candidate inside this sandbox with the network namespace unshared — which is what lets it report "no provider was called" as a property of the run rather than a claim.

## Why the sentence was replaced rather than deleted
`AGENTS.md` carried "Consumer: none yet — no repository beside this one names `b10x-sandbox`", from a `grep -rlw` over the sibling repositories on 2026-09-15 that matched only that day's org-state review pages. It was true when written and is now false. The replacement records both, because a claim that changed is worth more than a claim that was quietly corrected.

## Not in scope
The catalog row. ADR 0052:25 declined the consumer condition outright, and ADR 0060:97-99 names the real blocker — `atlas docs reconcile` failing on an unrelated AEP documentation error while AEP is frozen. Nothing here touches that.

## Acceptance
`AGENTS.md` names a consumer that exists, and the named invocation sites resolve.
