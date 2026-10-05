---
title: Getting started
sidebar_position: 2
description: Install b10x-sandbox from source, then watch what a sandboxed command can and cannot see.
lede: Install the command line from the repository, run a command in a sandbox, mirror a second directory beside it, and print the exact bubblewrap argument list.
source: src/main.rs, src/cli.rs, src/layout.rs, tests/bubblewrap.rs; every command below was run with b10x-sandbox 0.1.0 and bubblewrap 0.12.0
---

# Getting started

You need Linux, [bubblewrap](https://github.com/containers/bubblewrap) on `PATH` (`bwrap
--version`) and a Rust toolchain, 1.97 or newer. Nothing is on a registry and nothing is released
yet, so you build from the repository.

## Install

```console
$ cargo install --locked --git https://github.com/beyond10x/sandbox b10x-sandbox
$ b10x-sandbox --version
b10x-sandbox 0.1.0
```

From a checkout, `task install` does the same from the working tree.

## Make a small project

Two sibling directories and a file beside them that the sandbox should not see:

```console
$ mkdir -p ~/.cache/sandbox-demo/app ~/.cache/sandbox-demo/lib
$ echo 'hello from lib' > ~/.cache/sandbox-demo/lib/note.txt
$ echo private > ~/.cache/sandbox-demo/secret.txt
$ cd ~/.cache/sandbox-demo/app
```

## Run one command

Everything after `--` is the command. Without one, `b10x-sandbox` starts your `$SHELL`.

```console
$ b10x-sandbox -- sh -c 'pwd; ls /; ls /home'
/workspace
bin
dev
lib
lib64
proc
tmp
usr
workspace
ls: cannot access '/home': No such file or directory
```

The project directory is `/workspace`. `/etc`, `/home` and the rest of the host are not mounted, so
a path outside the project does not resolve at all: `cat ../secret.txt` fails with "No such file or
directory". The exit status is the command's, here 2 from the last `ls`.

What the command writes in `/workspace` is written in the project directory on the host:

```console
$ b10x-sandbox -- sh -c 'echo hi > made-inside'
$ ls
made-inside
```

## No network

```console
$ b10x-sandbox -- ip -brief link
lo               UNKNOWN        00:00:00:00:00:00 <LOOPBACK,UP,LOWER_UP>
```

The sandbox has its own network namespace with only a loopback interface. `--net` keeps the host's.

## Mirror a second directory

`--dir` adds a writable directory. The common ancestor of the working directory and every `--dir`
becomes `/workspace`, and each directory appears at its path relative to it:

```console
$ b10x-sandbox --dir ../lib -- sh -c 'pwd; ls /workspace; cat ../lib/note.txt; touch /workspace/new'
/workspace/app
app
lib
hello from lib
touch: cannot touch '/workspace/new': Read-only file system
```

`../lib` resolves exactly as on the host. `secret.txt` sits in the ancestor but is not there: only
the named directories are bound, and `/workspace` itself takes no writes.
[The workspace layout](./concepts/workspace-layout.md) explains the rule.

## A path that does not fit is refused

```console
$ b10x-sandbox --dir ..
b10x-sandbox: ~/.cache/sandbox-demo/app: nested inside ~/.cache/sandbox-demo, already writable through it
$ echo $?
2
```

(The tool prints absolute paths; `~` stands for your home directory here.) Nothing ran. A missing,
duplicated, nested or symlinked path is refused by name and never adjusted.

## See the exact argument list

`--dry-run` prints the program and its arguments, one per line, and runs nothing:

```console
$ b10x-sandbox --dry-run --dir ../lib | sed "s|$HOME|~|" | grep -A2 -- '--bind\|--remount-ro'
--bind
~/.cache/sandbox-demo/app
/workspace/app
--bind
~/.cache/sandbox-demo/lib
/workspace/lib
--remount-ro
/workspace
--chdir
```

The whole list for both backends is in [Argument lists](./reference/argv.md).

## Next

- [Run a project's tests in the sandbox](./guides/run-tests-in-the-sandbox.md), with a toolchain
  read-only.
- [Use the Docker backend](./guides/use-the-docker-backend.md).
- [Embed the library](./guides/embed-the-library.md) in a Rust program.
