# Changelog

Every change a user of `b10x-sandbox` or the `b10x_sandbox` library would notice. Nothing is
released yet; everything below is on `main`.

## Unreleased

### Added

- `b10x-sandbox`: respawn `$SHELL`, or the command after `--`, inside bubblewrap with the working
  directory writable at `/workspace` and the rest of the host absent apart from `/usr`, `/bin`,
  `/lib` and `/lib64`. No network unless `--net`.
- `--dir` mirrors further writable directories under `/workspace` at their paths relative to the
  common ancestor; `--ro` binds a directory read-only at its own path.
- `--backend docker` and `--image`: the same layout through `docker run`, as the caller's uid, with
  no capabilities, no network and a read-only image root.
- `--dry-run` prints the program and its argument list; `--allow-tiocsti` runs when
  `dev.tty.legacy_tiocsti` is 1, which is otherwise refused.
- Named refusals, exit 2: an input path that is not absolute, not canonical, not a directory,
  duplicated or nested; a missing `bwrap` or `docker`, never a fallback.
- The `b10x_sandbox` library: `Layout`, `Confinement`, `Options`, `Backend`, and the `docker`
  argument list.
- `b10x_sandbox::cli`: the clap definition of the command line, public so the documentation
  generator can walk it.
- The documentation site source in `website/`, with its generated CLI, argument-list, crate and
  status pages (`sandbox-docs`).

### Changed

- `--dry-run`'s help says it prints the program and its argument list, which under
  `--backend docker` is `docker`, not `bwrap`.

### Fixed

- Without `--dir`, `/workspace` was remounted read-only over the one writable bind, so nothing in
  the working directory could be written. The default layout is no longer remounted (`b95ab2c`).
