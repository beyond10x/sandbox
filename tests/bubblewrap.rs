//! Observes the confinement through a real bubblewrap, when one is on `PATH`.
//!
//! Absent bubblewrap this test prints `bwrap absent: confinement not observed` and returns: it
//! does not claim the confinement held, and a reader of the output can see it did not run.

use std::fs;
use std::process::Command;

use b10x_sandbox::{Confinement, Layout, Options};

fn bubblewrap_present() -> bool {
    Command::new("bwrap")
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success())
}

#[test]
fn mirrored_directories_resolve_against_each_other_and_nothing_else_is_visible() {
    if !bubblewrap_present() {
        println!("bwrap absent: confinement not observed");
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
    assert_eq!(layout.cwd, std::path::PathBuf::from("/workspace/x/a"));
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
    let confinement =
        Confinement::new(layout, Options::default(), ["/bin/sh", "-c", script]).unwrap();
    let output = confinement.command().unwrap().output().expect("bwrap runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(
        lines,
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
