//! The `docker run` argument list for a [`Layout`].
//!
//! As close to the bubblewrap posture as `docker run` allows, with the differences a reader should
//! be able to find: the container runs as the caller's uid and gid rather than in a user
//! namespace; `/workspace` is a tmpfs at mode `0555`, which is what makes the synthesized parents
//! read-only for a non-root user — the default layout binds the working directory at `/workspace`
//! itself, has no synthesized parents, and takes no tmpfs, since Docker refuses two mounts at one
//! destination; the environment is set with `-e` on top of the image's own
//! variables, since Docker has no clear-environment step; and there is no TIOCSTI concern because
//! Docker allocates its own pseudo-terminal.

use std::ffi::{OsStr, OsString};
use std::os::unix::fs::MetadataExt as _;

use crate::confinement::Options;
use crate::layout::{Layout, WORKSPACE};

/// The image used when a caller names none.
pub const DEFAULT_IMAGE: &str = "debian:stable-slim";

/// The uid and gid the container runs as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identity {
    pub uid: u32,
    pub gid: u32,
}

/// The calling process's own uid and gid, read from the owner of `/proc/self`.
///
/// # Errors
///
/// `/proc` is not mounted, which on Linux means the caller is already inside something odd.
pub fn caller_identity() -> std::io::Result<Identity> {
    let metadata = std::fs::metadata("/proc/self")?;
    Ok(Identity {
        uid: metadata.uid(),
        gid: metadata.gid(),
    })
}

/// The complete argument list for `docker`, without the program name.
pub fn argv(
    layout: &Layout,
    options: &Options,
    image: &str,
    tty: bool,
    identity: Identity,
    command: &[OsString],
) -> Vec<OsString> {
    let mut argv: Vec<OsString> = Vec::new();
    let mut push = |arguments: &[&OsStr]| argv.extend(arguments.iter().map(|a| (*a).to_owned()));

    // 1. One-shot, interactive, and a terminal only when the caller has one.
    push(&[o("run"), o("--rm"), o("-i")]);
    if tty {
        push(&[o("-t")]);
    }
    // 2. The caller's identity, so bind mounts are owned by the process inside.
    push(&[
        o("--user"),
        o(&format!("{}:{}", identity.uid, identity.gid)),
    ]);
    // 3. No network unless asked.
    if !options.network {
        push(&[o("--network"), o("none")]);
    }
    // 4. Capabilities, privilege escalation, image root and a private tmp.
    push(&[
        o("--cap-drop"),
        o("ALL"),
        o("--security-opt"),
        o("no-new-privileges"),
    ]);
    push(&[o("--read-only"), o("--tmpfs"), o("/tmp")]);
    // 5. The workspace: a read-only tmpfs the binds hang under.
    //
    // Omitted when a mapping is bound at `/workspace` itself — the default layout, with no
    // `--dir`. There are no synthesized parents to make read-only then, and Docker refuses a
    // volume and a tmpfs at one destination ("Duplicate mount point"), so the run would not start.
    if !layout.binds_workspace_root() {
        push(&[
            o("--mount"),
            o(&format!(
                "type=tmpfs,destination={WORKSPACE},tmpfs-mode=0555"
            )),
        ]);
    }
    // 6. Read-only roots at their own path, then the writable mappings.
    for root in &options.read_only_roots {
        let mut spec = root.as_os_str().to_owned();
        spec.push(":");
        spec.push(root.as_os_str());
        spec.push(":ro");
        push(&[o("-v"), &spec]);
    }
    for mapping in &layout.mappings {
        let mut spec = mapping.host.as_os_str().to_owned();
        spec.push(":");
        spec.push(mapping.mount.as_os_str());
        push(&[o("-v"), &spec]);
    }
    // 7. Working directory.
    push(&[o("-w"), layout.cwd.as_os_str()]);
    // 8. The same baseline the bubblewrap backend sets.
    for (name, value) in [
        ("PATH", "/usr/bin:/bin"),
        ("HOME", "/tmp"),
        ("LANG", "C.UTF-8"),
        ("LC_ALL", "C.UTF-8"),
        ("TZ", "UTC"),
    ] {
        push(&[o("-e"), o(&format!("{name}={value}"))]);
    }
    if let Some(term) = &options.term {
        let mut spec = OsString::from("TERM=");
        spec.push(term);
        push(&[o("-e"), &spec]);
    }
    for (name, value) in &options.env {
        let mut spec = name.clone();
        spec.push("=");
        spec.push(value);
        push(&[o("-e"), &spec]);
    }
    // 9. Image and command.
    push(&[o(image)]);
    let command = command.iter().map(OsString::as_os_str).collect::<Vec<_>>();
    push(&command);
    argv
}

fn o(s: &str) -> &OsStr {
    OsStr::new(s)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::PathBuf;

    use super::{Identity, argv, caller_identity};
    use crate::confinement::Options;
    use crate::layout::{Layout, Mapping};

    #[test]
    fn argv_for_a_two_directory_layout_with_a_root_is_exact() {
        let layout = Layout {
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
        };
        let options = Options {
            term: Some(OsString::from("xterm")),
            read_only_roots: vec![PathBuf::from("/home/me/.cargo")],
            ..Options::default()
        };
        let command = [OsString::from("/bin/sh"), OsString::from("-l")];
        let identity = Identity {
            uid: 1000,
            gid: 1000,
        };
        let got = argv(
            &layout,
            &options,
            "debian:stable-slim",
            true,
            identity,
            &command,
        )
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
        let expected: Vec<&str> = vec![
            "run",
            "--rm",
            "-i",
            "-t",
            "--user",
            "1000:1000",
            "--network",
            "none",
            "--cap-drop",
            "ALL",
            "--security-opt",
            "no-new-privileges",
            "--read-only",
            "--tmpfs",
            "/tmp",
            "--mount",
            "type=tmpfs,destination=/workspace,tmpfs-mode=0555",
            "-v",
            "/home/me/.cargo:/home/me/.cargo:ro",
            "-v",
            "/home/me/work/app:/workspace/app",
            "-v",
            "/home/me/work/lib:/workspace/lib",
            "-w",
            "/workspace/app",
            "-e",
            "PATH=/usr/bin:/bin",
            "-e",
            "HOME=/tmp",
            "-e",
            "LANG=C.UTF-8",
            "-e",
            "LC_ALL=C.UTF-8",
            "-e",
            "TZ=UTC",
            "-e",
            "TERM=xterm",
            "debian:stable-slim",
            "/bin/sh",
            "-l",
        ];
        assert_eq!(got, expected);
    }

    #[test]
    fn the_default_layout_takes_no_tmpfs_over_its_writable_bind() {
        // The default layout as `Layout::plan` computes it, not a hand-built one: the property
        // under test is a consequence of planning with no `--dir`, so planning is what is tested.
        let dir = tempfile::tempdir().expect("tempdir");
        let root = std::fs::canonicalize(dir.path()).expect("canonical tempdir");
        let layout = Layout::plan(&root, &[]).expect("layout");
        let command = [OsString::from("/bin/sh")];
        let identity = Identity {
            uid: 1000,
            gid: 1000,
        };
        let got = argv(
            &layout,
            &Options::default(),
            "debian:stable-slim",
            false,
            identity,
            &command,
        )
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
        assert!(
            !got.iter().any(|a| a.contains("destination=/workspace")),
            "a tmpfs beside the volume is refused as a duplicate mount point: {got:?}"
        );
        let volume = format!("{}:/workspace", root.display());
        assert!(
            got.windows(2).any(|w| w[0] == "-v" && w[1] == volume),
            "the working directory is the writable workspace: {got:?}"
        );
        assert!(
            got.windows(2).any(|w| w == ["-w", "/workspace"]),
            "the shell starts in the workspace it can write: {got:?}"
        );
    }

    #[test]
    fn caller_identity_is_this_process() {
        let identity = caller_identity().expect("/proc is mounted");
        assert_eq!(identity.uid, unsafe_uid());
    }

    /// The uid via the one other route std offers without libc: the owner of a file we create.
    fn unsafe_uid() -> u32 {
        use std::os::unix::fs::MetadataExt as _;
        let dir = tempfile::tempdir().unwrap();
        std::fs::metadata(dir.path()).unwrap().uid()
    }
}
