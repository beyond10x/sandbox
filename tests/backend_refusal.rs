//! Invariant 1 (`AGENTS.md`): bubblewrap is the default backend and Docker the second, and a
//! missing `bwrap` or `docker` is a named refusal — never a fallback to an unconfined process, and
//! never a fallback to the other backend.
//!
//! The refusal is observed by making the backend genuinely unreachable: the library test points
//! `Options::bubblewrap` at a path that does not exist, and the command-line tests run the binary
//! with a `PATH` holding one empty directory, so neither `bwrap` nor `docker` can be resolved
//! however the host is provisioned. Those run the binary as a child process on purpose — a
//! fallback would `exec` the command, and the marker file it would leave behind is the evidence
//! that it did. Nothing here needs `bwrap` or `docker` to be installed.
//!
//! Not exercised: the "Linux only" half of the invariant. These tests run on Linux and cannot
//! observe what the crate does anywhere else.

use std::fs;
use std::path::Path;
use std::process::Command;

use b10x_sandbox::{Backend, Confinement, ConfinementError, Layout, Options};

/// A canonical temporary tree, an empty directory to use as `PATH`, and the marker path a fallback
/// would create.
struct Host {
    _keep: tempfile::TempDir,
    root: std::path::PathBuf,
    empty_path: std::path::PathBuf,
    marker: std::path::PathBuf,
}

fn host() -> Host {
    let keep = tempfile::tempdir().expect("tempdir");
    let root = fs::canonicalize(keep.path()).expect("canonical tempdir");
    let empty_path = root.join("no-executables-here");
    fs::create_dir_all(&empty_path).expect("empty PATH directory");
    let marker = root.join("ran-outside-the-sandbox");
    Host {
        _keep: keep,
        root,
        empty_path,
        marker,
    }
}

/// The crate's binary with a `PATH` that resolves no program at all.
fn binary_with_no_programs_on_path(host: &Host) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_b10x-sandbox"));
    command.env("PATH", &host.empty_path);
    command.current_dir(&host.root);
    command
}

/// The first line of `--dry-run` output, which is the program the run would have `exec`ed.
fn dry_run_program(cwd: &Path, extra: &[&str]) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_b10x-sandbox"));
    let output = command
        .current_dir(cwd)
        .arg("--dry-run")
        .args(extra)
        .output()
        .expect("the binary runs");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("utf-8")
        .lines()
        .next()
        .expect("a program line")
        .to_owned()
}

#[test]
fn bubblewrap_is_the_default_backend() {
    assert_eq!(Backend::default(), Backend::Bubblewrap);
    assert_eq!(Options::default().backend, Backend::Bubblewrap);

    let host = host();
    let layout = Layout::plan(&host.root, &[]).expect("layout");
    let confinement =
        Confinement::new(layout, Options::default(), ["/bin/sh"]).expect("a valid confinement");
    assert_eq!(confinement.backend(), &Backend::Bubblewrap);
    assert_eq!(confinement.program(), Path::new("bwrap"));
}

#[test]
fn the_command_line_runs_bubblewrap_by_default_and_docker_only_when_asked() {
    let host = host();
    assert_eq!(dry_run_program(&host.root, &[]), "bwrap");
    assert_eq!(
        dry_run_program(&host.root, &["--backend", "docker"]),
        "docker"
    );
}

#[test]
fn the_backend_flag_offers_exactly_bubblewrap_and_docker() {
    // An unconfined third choice would have to appear here first.
    let output = Command::new(env!("CARGO_BIN_EXE_b10x-sandbox"))
        .args(["--backend", "none", "--dry-run"])
        .output()
        .expect("the binary runs");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("utf-8");
    assert!(
        stderr.contains("[possible values: bubblewrap, docker]"),
        "stderr: {stderr}"
    );
}

#[test]
fn a_missing_bubblewrap_is_a_named_refusal_that_names_that_binary() {
    let host = host();
    let absent = host.root.join("absent-bwrap");
    let layout = Layout::plan(&host.root, &[]).expect("layout");
    let options = Options {
        // The terminal guard runs before the exec and would mask the refusal under test on a host
        // whose `legacy_tiocsti` sysctl reads 1.
        allow_tiocsti: true,
        bubblewrap: absent.clone(),
        ..Options::default()
    };
    // A command that exits at once rather than an interactive shell: `run` execs, so if a
    // regression ever made it reach *any* program this process is replaced by it, and a shell
    // waiting on stdin would hang the run instead of failing it.
    let command = ["/bin/sh", "-c", "exit 97"];
    let confinement = Confinement::new(layout, options, command).expect("a valid confinement");
    match confinement.run() {
        Err(ConfinementError::ProgramMissing(program)) => {
            assert_eq!(
                program, absent,
                "the refusal must name the backend that is missing"
            );
        }
        Err(other) => panic!("expected a named refusal, got: {other}"),
        Ok(never) => match never {},
    }
}

#[test]
fn a_missing_bubblewrap_never_falls_back_to_running_the_command_unconfined() {
    let host = host();
    // `: >` is a shell builtin redirection, so the marker appears even though `PATH` here
    // resolves no external program: the evidence must not depend on `touch` being findable.
    let script = format!(": > {}", host.marker.display());
    let output = binary_with_no_programs_on_path(&host)
        .args(["--allow-tiocsti", "--", "/bin/sh", "-c", script.as_str()])
        .output()
        .expect("the binary runs");

    let stderr = String::from_utf8(output.stderr).expect("utf-8");
    assert_eq!(output.status.code(), Some(2), "stderr: {stderr}");
    assert_eq!(stderr.trim(), "b10x-sandbox: bwrap not found");
    assert!(
        !host.marker.exists(),
        "the command ran with no sandbox around it; stderr: {stderr}"
    );
}

#[test]
fn a_missing_docker_never_falls_back_to_bubblewrap_or_to_an_unconfined_command() {
    let host = host();
    // `: >` is a shell builtin redirection, so the marker appears even though `PATH` here
    // resolves no external program: the evidence must not depend on `touch` being findable.
    let script = format!(": > {}", host.marker.display());
    let output = binary_with_no_programs_on_path(&host)
        .args([
            "--backend",
            "docker",
            "--",
            "/bin/sh",
            "-c",
            script.as_str(),
        ])
        .output()
        .expect("the binary runs");

    let stderr = String::from_utf8(output.stderr).expect("utf-8");
    assert_eq!(output.status.code(), Some(2), "stderr: {stderr}");
    assert_eq!(stderr.trim(), "b10x-sandbox: docker not found");
    assert!(
        !stderr.contains("bwrap"),
        "the chosen backend was Docker; the other one is not a fallback. stderr: {stderr}"
    );
    assert!(
        !host.marker.exists(),
        "the command ran with no sandbox around it; stderr: {stderr}"
    );
}
