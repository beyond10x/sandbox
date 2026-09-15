//! The command line over the library: dry-run output and a named refusal.

use std::fs;
use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_b10x-sandbox"))
}

#[test]
fn dry_run_prints_the_mirrored_argv_and_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(dir.path()).unwrap();
    let a = root.join("x/a");
    let b = root.join("y/z/b");
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();

    let output = binary()
        .current_dir(&a)
        .args(["--dry-run", "--dir", "../../y/z/b", "--", "/bin/sh"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines[0], "bwrap");
    let chdir = lines.iter().position(|l| *l == "--chdir").unwrap();
    assert_eq!(lines[chdir + 1], "/workspace/x/a");
    let binds = lines
        .windows(3)
        .filter(|w| w[0] == "--bind")
        .map(|w| (w[1].to_owned(), w[2].to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(
        binds,
        vec![
            (
                a.to_string_lossy().into_owned(),
                "/workspace/x/a".to_owned()
            ),
            (
                b.to_string_lossy().into_owned(),
                "/workspace/y/z/b".to_owned()
            ),
        ]
    );
    assert_eq!(lines.last(), Some(&"/bin/sh"));
}

#[test]
fn a_missing_directory_is_refused_with_exit_two() {
    let dir = tempfile::tempdir().unwrap();
    let output = binary()
        .current_dir(dir.path())
        .args(["--dry-run", "--dir", "does-not-exist"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("b10x-sandbox: "), "stderr: {stderr}");
    assert!(
        stderr.contains("not an existing directory"),
        "stderr: {stderr}"
    );
}

#[test]
fn docker_backend_dry_run_prints_docker_and_the_mirrored_volumes() {
    let dir = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(dir.path()).unwrap();
    let a = root.join("x/a");
    let b = root.join("y/z/b");
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();

    let output = binary()
        .current_dir(&a)
        .args(["--backend", "docker", "--dry-run", "--dir", "../../y/z/b"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines[0], "docker");
    assert_eq!(lines[1], "run");
    let workdir = lines.iter().position(|l| *l == "-w").unwrap();
    assert_eq!(lines[workdir + 1], "/workspace/x/a");
    let volumes = lines
        .windows(2)
        .filter(|w| w[0] == "-v")
        .map(|w| w[1].to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        volumes,
        vec![
            format!("{}:/workspace/x/a", a.display()),
            format!("{}:/workspace/y/z/b", b.display())
        ]
    );
    assert_eq!(lines.last(), Some(&"/bin/sh"));
}
