//! Observes the Docker backend through a real daemon, when one answers.
//!
//! Absent Docker this test prints `docker absent: confinement not observed` and returns: it does
//! not claim the confinement held.

use std::ffi::OsString;
use std::fs;
use std::process::Command;

use b10x_sandbox::docker::{argv, caller_identity};
use b10x_sandbox::{Layout, Options};

fn docker_present() -> bool {
    Command::new("docker")
        .arg("version")
        .output()
        .is_ok_and(|out| out.status.success())
}

#[test]
fn mirrored_directories_resolve_against_each_other_and_nothing_else_is_visible() {
    if !docker_present() {
        println!("docker absent: confinement not observed");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(dir.path()).unwrap();
    let a = root.join("x/a");
    let b = root.join("y/z/b");
    for path in [&a, &b, &root.join("x/other"), &root.join("secret")] {
        fs::create_dir_all(path).unwrap();
    }
    fs::write(root.join("secret/token"), b"do-not-read").unwrap();

    let layout = Layout::plan(&a, std::slice::from_ref(&b)).unwrap();
    let script = concat!(
        "echo cwd=$(pwd);",
        "echo top=$(ls /workspace | tr '\\n' ' ');",
        "echo x=$(ls /workspace/x | tr '\\n' ' ');",
        "touch ../../y/z/b/probe && echo relative-write=ok;",
        "touch ../.parent 2>/dev/null && echo parent-write=ok || echo parent-write=refused;",
        "touch /workspace/.root 2>/dev/null && echo root-write=ok || echo root-write=refused;",
        "cat /workspace/secret/token 2>/dev/null || echo secret=absent;",
        "echo home=$HOME",
    );
    // `docker::argv` directly, not through `Confinement`: this test belongs to the story that
    // owns the argument list, and must pass before the backend is selectable.
    let command = ["/bin/sh", "-c", script].map(OsString::from);
    let identity = caller_identity().unwrap();
    let arguments = argv(
        &layout,
        &Options::default(),
        "debian:stable-slim",
        false,
        identity,
        &command,
    );
    let output = Command::new("docker")
        .args(arguments)
        .output()
        .expect("docker runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        vec![
            "cwd=/workspace/x/a",
            "top=x y",
            "x=a",
            "relative-write=ok",
            "parent-write=refused",
            "root-write=refused",
            "secret=absent",
            "home=/tmp",
        ],
        "full stdout:\n{stdout}"
    );
    assert!(
        b.join("probe").is_file(),
        "the relative write reached the host directory"
    );
}

/// The default layout under Docker, checked separately as
/// `atlas/reviews/org-state/2026-09-15-full-01/findings.md` § ORG-0002 asks: the review exercised
/// only the bubblewrap default. Docker refuses a tmpfs and a volume at one destination, so the
/// workspace tmpfs is omitted when a mapping is bound at `/workspace` itself; without that this
/// run fails to start rather than running read-only.
#[test]
fn the_default_workspace_is_writable_and_the_undeclared_host_stays_out() {
    if !docker_present() {
        println!("docker absent: confinement not observed");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(dir.path()).unwrap();
    let app = root.join("app");
    let sibling = root.join("sibling");
    let toolchain = root.join("toolchain");
    for path in [&app, &sibling, &toolchain] {
        fs::create_dir_all(path).unwrap();
    }
    fs::write(sibling.join("token"), b"do-not-read").unwrap();
    fs::write(toolchain.join("tool"), b"toolchain-file").unwrap();

    let layout = Layout::plan(&app, &[]).unwrap();
    let script = format!(
        concat!(
            "echo cwd=$(pwd);",
            "test -w /workspace && echo predicate=workspace-writable || echo predicate=workspace-read-only;",
            "printf sandbox-wrote > probe && echo write=ok || echo write=refused;",
            "echo read=$(cat /workspace/probe);",
            "rm probe && echo remove=ok || echo remove=refused;",
            "test -e probe && echo removed=no || echo removed=yes;",
            "printf kept > /workspace/kept && echo kept-write=ok || echo kept-write=refused;",
            "cat {sibling}/token 2>/dev/null || echo sibling=absent;",
            "echo tool=$(cat {toolchain}/tool);",
            "touch {toolchain}/intruder 2>/dev/null && echo ro-write=ok || echo ro-write=refused",
        ),
        sibling = sibling.display(),
        toolchain = toolchain.display(),
    );
    let options = Options {
        read_only_roots: vec![toolchain.clone()],
        ..Options::default()
    };
    let command = ["/bin/sh", "-c", script.as_str()].map(OsString::from);
    let identity = caller_identity().unwrap();
    let arguments = argv(
        &layout,
        &options,
        "debian:stable-slim",
        false,
        identity,
        &command,
    );
    let output = Command::new("docker")
        .args(arguments)
        .output()
        .expect("docker runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        vec![
            "cwd=/workspace",
            "predicate=workspace-writable",
            "write=ok",
            "read=sandbox-wrote",
            "remove=ok",
            "removed=yes",
            "kept-write=ok",
            "sibling=absent",
            "tool=toolchain-file",
            "ro-write=refused",
        ],
        "full stdout:\n{stdout}"
    );
    assert_eq!(
        fs::read_to_string(app.join("kept")).expect("the write reached the host directory"),
        "kept"
    );
    assert!(
        !app.join("probe").exists(),
        "the remove reached the host directory too"
    );
    assert!(
        !toolchain.join("intruder").exists(),
        "a read-only root is never written through"
    );
}
