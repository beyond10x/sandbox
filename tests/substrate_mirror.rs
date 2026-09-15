//! Invariant 4 (`AGENTS.md`): the bubblewrap isolation argument list in `src/confinement.rs`
//! mirrors substrate's, `src/docker.rs` is its closest `docker run` equivalent, and the only
//! differences are the ones each module's doc comment names.
//!
//! Why this is not the exact-argv tests over again: those compare the produced list with a copy of
//! itself written in the same commit, so dropping an argument *and* editing the expectation to
//! match passes them together — which is precisely the "cleanup" invariant 4 forbids. The
//! expectation here is an independent transcription of substrate's own source, so that same edit
//! fails here.
//!
//! The third difference each module names — how `/workspace` is built, and that the default layout
//! takes neither a remount nor a tmpfs — is covered by the unit tests in those modules and by
//! `tests/bubblewrap.rs` and `tests/docker.rs`, and is not repeated here.

use std::ffi::OsString;
use std::path::PathBuf;

use b10x_sandbox::confinement::LEGACY_TIOCSTI_SYSCTL;
use b10x_sandbox::docker::{self, Identity};
use b10x_sandbox::{Confinement, Layout, Mapping, Options};

/// One row of the mirror: an argument of substrate's isolation set with its operands, and the
/// `docker run` argument that stands in for it, or `None` when nothing in the `docker run` list
/// mirrors it.
type Row = (&'static [&'static str], Option<&'static [&'static str]>);

/// Substrate's isolation set, transcribed from `crates/substrate-host/src/process.rs` at substrate
/// 0.7.x: `USER_NAMESPACE_ARGV` (line 62), the `command.args` block that follows it (lines
/// 1911-1938), and the `--seccomp` argument after that block (line 1939). One row per argument, in
/// substrate's order. `--seccomp` carries a file descriptor substrate computes at spawn time, so
/// only the flag is transcribed.
///
/// The second column is what `docker run` uses instead. `None` is a deliberate entry, not a gap:
/// the container runtime supplies that property without an argument — its own IPC, PID and UTS
/// namespaces, its own `/proc` and `/dev`, its own default seccomp profile — so there is no
/// argument to compare and asserting one would invent an equivalence the argv does not contain.
const MIRROR: &[Row] = &[
    // No user namespace under Docker: the container runs as the caller's uid and gid instead,
    // which is the first difference `src/docker.rs` names.
    (&["--unshare-user"], Some(&["--user", "1000:1000"])),
    (&["--disable-userns"], None),
    (&["--unshare-ipc"], None),
    (&["--unshare-pid"], None),
    (&["--unshare-net"], Some(&["--network", "none"])),
    (&["--unshare-uts"], None),
    // Difference 1 of `src/confinement.rs`: dropped so an interactive shell keeps its controlling
    // terminal, against the `legacy_tiocsti` guard checked below.
    (&["--new-session"], None),
    (&["--die-with-parent"], Some(&["--rm"])),
    // Docker has no clear-environment step; the baseline is set with `-e` on top of the image's
    // own variables. Asserted by `both_backends_set_the_same_environment_baseline`.
    (&["--clearenv"], None),
    (&["--ro-bind", "/usr", "/usr"], Some(&["--read-only"])),
    (&["--ro-bind-try", "/bin", "/bin"], None),
    (&["--ro-bind-try", "/lib", "/lib"], None),
    (&["--ro-bind-try", "/lib64", "/lib64"], None),
    (&["--proc", "/proc"], None),
    (&["--dev", "/dev"], None),
    (&["--tmpfs", "/tmp"], Some(&["--tmpfs", "/tmp"])),
    // Difference 2 of `src/confinement.rs`: substrate's socket-family filter is not carried yet.
    (&["--seccomp"], None),
];

/// The two differences `src/confinement.rs` admits against substrate, in substrate's order.
const ADMITTED_DIFFERENCES: [&str; 2] = ["--new-session", "--seccomp"];

/// Hardening the `docker run` list carries that has no argument in substrate's bubblewrap set,
/// because bubblewrap gets the same effect from the user namespace it unshares.
const DOCKER_ONLY: &[&[&str]] = &[
    &["--cap-drop", "ALL"],
    &["--security-opt", "no-new-privileges"],
];

/// The environment both backends set, `--setenv NAME VALUE` against `-e NAME=VALUE`.
const BASELINE_ENV: [(&str, &str); 5] = [
    ("PATH", "/usr/bin:/bin"),
    ("HOME", "/tmp"),
    ("LANG", "C.UTF-8"),
    ("LC_ALL", "C.UTF-8"),
    ("TZ", "UTC"),
];

fn strings(argv: &[OsString]) -> Vec<String> {
    argv.iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect()
}

/// Whether `group` appears as consecutive arguments of `argv`.
fn contains(argv: &[String], group: &[&str]) -> bool {
    argv.windows(group.len())
        .any(|window| window.iter().zip(group).all(|(a, b)| a.as_str() == *b))
}

/// The arguments of substrate's set that the bubblewrap list does not carry, named by their flag.
fn absent_from_the_bubblewrap_list(argv: &[String]) -> Vec<&'static str> {
    MIRROR
        .iter()
        .filter(|(substrate, _)| !contains(argv, substrate))
        .map(|(substrate, _)| substrate[0])
        .collect()
}

/// The arguments of substrate's set whose `docker run` equivalent is missing from the Docker list.
fn without_a_docker_equivalent(argv: &[String]) -> Vec<&'static str> {
    MIRROR
        .iter()
        .filter(|(_, equivalent)| equivalent.is_some_and(|group| !contains(argv, group)))
        .map(|(substrate, _)| substrate[0])
        .collect()
}

/// Two directories under a common ancestor: the mirrored layout, with synthesized parents.
fn mirrored_layout() -> Layout {
    Layout {
        ancestor: PathBuf::from("/home/me/work"),
        cwd: PathBuf::from("/workspace/app"),
        mappings: vec![
            Mapping {
                host: "/home/me/work/app".into(),
                mount: "/workspace/app".into(),
            },
            Mapping {
                host: "/home/me/work/lib".into(),
                mount: "/workspace/lib".into(),
            },
        ],
    }
}

fn bubblewrap_argv(layout: Layout, network: bool) -> Vec<String> {
    let options = Options {
        network,
        ..Options::default()
    };
    let confinement = Confinement::new(layout, options, ["/bin/sh"]).expect("a valid confinement");
    strings(
        &confinement
            .argv()
            .expect("the bubblewrap list needs nothing"),
    )
}

fn docker_argv(layout: &Layout, network: bool) -> Vec<String> {
    let options = Options {
        network,
        ..Options::default()
    };
    let command = [OsString::from("/bin/sh")];
    strings(&docker::argv(
        layout,
        &options,
        docker::DEFAULT_IMAGE,
        false,
        Identity {
            uid: 1000,
            gid: 1000,
        },
        &command,
    ))
}

#[test]
fn the_bubblewrap_list_differs_from_substrates_only_where_its_module_doc_says() {
    let argv = bubblewrap_argv(mirrored_layout(), false);
    assert_eq!(
        absent_from_the_bubblewrap_list(&argv),
        ADMITTED_DIFFERENCES,
        "substrate's isolation set minus this list must be exactly the differences \
         `src/confinement.rs` documents; dropping an argument is a design change for the planning \
         store, not a cleanup. Full argv: {argv:?}"
    );
}

#[test]
fn the_default_layout_keeps_the_whole_isolation_set_too() {
    // Planned, not hand-built: the default mapping is the one the `/workspace` fix changed, and
    // the isolation set must have survived it.
    let dir = tempfile::tempdir().expect("tempdir");
    let root = std::fs::canonicalize(dir.path()).expect("canonical tempdir");
    let layout = Layout::plan(&root, &[]).expect("layout");
    let argv = bubblewrap_argv(layout, false);
    assert_eq!(
        absent_from_the_bubblewrap_list(&argv),
        ADMITTED_DIFFERENCES,
        "full argv: {argv:?}"
    );
}

#[test]
fn keeping_the_host_network_drops_unshare_net_and_nothing_else() {
    let argv = bubblewrap_argv(mirrored_layout(), true);
    assert_eq!(
        absent_from_the_bubblewrap_list(&argv),
        ["--unshare-net", "--new-session", "--seccomp"],
        "`--net` is the caller's one permitted subtraction. Full argv: {argv:?}"
    );
}

#[test]
fn the_terminal_guard_that_stands_in_for_new_session_is_still_there() {
    // `--new-session` is admitted as absent only because this guard replaces it, so the absence
    // and the guard are asserted together: dropping the guard makes the difference undocumented.
    assert_eq!(LEGACY_TIOCSTI_SYSCTL, "/proc/sys/dev/tty/legacy_tiocsti");
    assert!(
        !Options::default().allow_tiocsti,
        "the guard is on unless a caller asks for it off"
    );
}

#[test]
fn the_docker_list_carries_an_equivalent_for_every_substrate_argument_that_has_one() {
    let layout = mirrored_layout();
    let argv = docker_argv(&layout, false);
    assert!(
        without_a_docker_equivalent(&argv).is_empty(),
        "no `docker run` equivalent left for {:?}. Full argv: {argv:?}",
        without_a_docker_equivalent(&argv)
    );
}

#[test]
fn keeping_the_host_network_under_docker_drops_the_network_argument_and_nothing_else() {
    let layout = mirrored_layout();
    let argv = docker_argv(&layout, true);
    assert_eq!(
        without_a_docker_equivalent(&argv),
        ["--unshare-net"],
        "full argv: {argv:?}"
    );
}

#[test]
fn the_docker_list_keeps_the_hardening_that_has_no_bubblewrap_twin() {
    let layout = mirrored_layout();
    let argv = docker_argv(&layout, false);
    for group in DOCKER_ONLY {
        assert!(
            contains(&argv, group),
            "{group:?} is what stands in for the user namespace bubblewrap unshares. \
             Full argv: {argv:?}"
        );
    }
}

#[test]
fn both_backends_set_the_same_environment_baseline() {
    let bubblewrap = bubblewrap_argv(mirrored_layout(), false);
    let layout = mirrored_layout();
    let docker = docker_argv(&layout, false);
    for (name, value) in BASELINE_ENV {
        assert!(
            contains(&bubblewrap, &["--setenv", name, value]),
            "bubblewrap dropped {name}. Full argv: {bubblewrap:?}"
        );
        assert!(
            contains(&docker, &["-e", &format!("{name}={value}")]),
            "docker dropped {name}. Full argv: {docker:?}"
        );
    }
}
