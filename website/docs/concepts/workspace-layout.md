---
title: The workspace layout
sidebar_position: 2
description: How b10x-sandbox decides where each host directory appears under /workspace, and which input paths it refuses.
lede: The longest common ancestor of the working directory and every --dir becomes /workspace; each directory is bound at its path relative to it, and nothing in between is visible.
source: src/layout.rs and its unit tests, tests/bubblewrap.rs, tests/docker.rs
---

# The workspace layout

`Layout::plan(cwd, dirs)` is a pure function of paths. It decides where every writable directory
lands and where the command starts; the backends only turn the result into mounts.

## Without `--dir`

The working directory is bound writable at `/workspace` itself, and the command starts there.

| Host | In the sandbox |
|---|---|
| `~/work/app` (working directory) | `/workspace` |

## With `--dir`

The longest common ancestor of the working directory and every `--dir` becomes `/workspace`. Each
input is bound at its path relative to that ancestor, and the command starts at the working
directory's place.

| Host | In the sandbox |
|---|---|
| `~/work` (ancestor, not bound) | `/workspace`, an empty read-only directory |
| `~/work/app` (working directory) | `/workspace/app`, writable; the command starts here |
| `~/work/lib` (`--dir ../lib`) | `/workspace/lib`, writable |
| `~/work/secret.txt` | absent |

Relative references between the mapped directories (`../lib` from `app`) resolve as on the host,
because the relative positions are the same. The directories between the ancestor and an input are
synthesized empty by the backend and are read-only: under bubblewrap `/workspace` is a tmpfs
remounted read-only after the binds, under Docker a tmpfs of mode `0555`. A file can be created in
a bound directory and not beside it.

Inputs under different top-level directories (`/srv/a` and `/opt/b`) have `/` as their ancestor and
appear as `/workspace/srv/a` and `/workspace/opt/b`.

`--ro` is separate from the layout: a read-only directory is bound at its own absolute path, not
under `/workspace`.

## Refusals

Every input is checked before anything is computed, and the first failure is returned; nothing is
adjusted.

| Input | Message |
|---|---|
| a relative path (library callers; the command line joins it onto the working directory first) | `<path>: not an absolute path` |
| a symlink, `.` or `..` in a component | `` <path>: not canonical (a symlink, `.` or `..` in a component) `` |
| a missing path, or a file | `<path>: not an existing directory` |
| the same directory twice | `<path>: listed twice` |
| a directory inside another input | `<inner>: nested inside <outer>, already writable through it` |

The command line resolves `.` and `..` in a relative `--dir` or `--ro` lexically, so `../lib`
works, while a symlink anywhere in the result is still refused. A `--ro` path gets the same
absolute, canonical and directory checks.
