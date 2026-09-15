//! The bubblewrap argument list, and running it.
//!
//! The isolation set below is substrate's (`crates/substrate-host/src/process.rs:1911-1937` at
//! substrate 0.7.0), with three deliberate differences a reader should be able to find:
//!
//! - **no `--new-session`**: an interactive shell needs its controlling terminal. The hazard that
//!   flag closes, a sandboxed process injecting input into the terminal through `TIOCSTI`, is
//!   closed on kernels 6.2 and later by `dev.tty.legacy_tiocsti = 0`; [`Confinement::run`] reads
//!   the sysctl and refuses when it is `1` unless [`Options::allow_tiocsti`] is set;
//! - **no seccomp filter**: substrate's socket-family filter is not carried yet;
//! - **`/workspace` is a tmpfs with one bind per mapped directory**, remounted read-only so the
//!   synthesized parents are not writable, where substrate binds one workspace directory. The
//!   default layout has no synthesized parents — the working directory is bound at `/workspace`
//!   itself — and is not remounted, or the one directory the caller asked for would be read-only.

use std::convert::Infallible;
use std::ffi::{OsStr, OsString};
use std::io;
use std::os::unix::process::CommandExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;

use crate::docker;
use crate::layout::{Layout, LayoutError, WORKSPACE, canonical_directory};

/// The sysctl that decides whether omitting `--new-session` is safe.
pub const LEGACY_TIOCSTI_SYSCTL: &str = "/proc/sys/dev/tty/legacy_tiocsti";

/// Which program confines the process.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Backend {
    /// `bwrap`, the default: the argument list documented at the top of this module.
    #[default]
    Bubblewrap,
    /// `docker run` from `image`; see [`crate::docker`] for the posture and its differences.
    Docker {
        /// The image to run, such as [`docker::DEFAULT_IMAGE`].
        image: String,
        /// Allocate a terminal (`-t`); set when the caller's stdin is one.
        tty: bool,
    },
}

/// What a caller may vary. Everything not here is the fixed isolation set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Which program confines the process. Default bubblewrap.
    pub backend: Backend,
    /// Keep the host network namespace. Default `false`: `--unshare-net`.
    pub network: bool,
    /// Run even when `dev.tty.legacy_tiocsti` is `1`. Default `false`.
    pub allow_tiocsti: bool,
    /// Host directories bound read-only at their own absolute path, such as a toolchain home.
    pub read_only_roots: Vec<PathBuf>,
    /// `TERM` for the sandboxed process; `None` sets none.
    pub term: Option<OsString>,
    /// Further environment pairs, set after the baseline and able to override it.
    pub env: Vec<(OsString, OsString)>,
    /// The bubblewrap binary. Default `bwrap`, resolved on `PATH` at exec time.
    pub bubblewrap: PathBuf,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            backend: Backend::Bubblewrap,
            network: false,
            allow_tiocsti: false,
            read_only_roots: Vec::new(),
            term: None,
            env: Vec::new(),
            bubblewrap: PathBuf::from("bwrap"),
        }
    }
}

/// Why a confinement could not be built or run.
#[derive(Debug, Error)]
pub enum ConfinementError {
    #[error("no command to run")]
    EmptyCommand,
    #[error("read-only root {0}")]
    ReadOnlyRoot(#[source] LayoutError),
    #[error(
        "{LEGACY_TIOCSTI_SYSCTL} is 1: a sandboxed process could inject terminal input; set it to 0 \
         or pass --allow-tiocsti"
    )]
    LegacyTiocsti,
    #[error("{0} not found")]
    ProgramMissing(PathBuf),
    #[error("could not read the caller's uid and gid from /proc/self: {0}")]
    Identity(#[source] io::Error),
    #[error("could not exec {0}: {1}")]
    Exec(PathBuf, #[source] io::Error),
}

/// A complete, validated bubblewrap invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Confinement {
    layout: Layout,
    options: Options,
    command: Vec<OsString>,
}

impl Confinement {
    /// Validate the read-only roots and the command; nothing touches the system yet.
    ///
    /// # Errors
    ///
    /// An empty command, or a read-only root that is not an absolute canonical directory.
    pub fn new<C, A>(
        layout: Layout,
        mut options: Options,
        command: C,
    ) -> Result<Self, ConfinementError>
    where
        C: IntoIterator<Item = A>,
        A: Into<OsString>,
    {
        let command = command.into_iter().map(Into::into).collect::<Vec<_>>();
        if command.is_empty() {
            return Err(ConfinementError::EmptyCommand);
        }
        options.read_only_roots = options
            .read_only_roots
            .iter()
            .map(|root| canonical_directory(root).map_err(ConfinementError::ReadOnlyRoot))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            layout,
            options,
            command,
        })
    }

    /// The layout this confinement binds.
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// The backend this confinement runs on.
    pub fn backend(&self) -> &Backend {
        &self.options.backend
    }

    /// The program [`Confinement::run`] will exec: the bubblewrap binary, or `docker`.
    pub fn program(&self) -> &Path {
        match &self.options.backend {
            Backend::Bubblewrap => &self.options.bubblewrap,
            Backend::Docker { .. } => Path::new("docker"),
        }
    }

    /// The complete argument list for [`Confinement::program`], without the program name.
    ///
    /// # Errors
    ///
    /// Under Docker, the caller's uid and gid could not be read.
    pub fn argv(&self) -> Result<Vec<OsString>, ConfinementError> {
        match &self.options.backend {
            Backend::Bubblewrap => Ok(self.bubblewrap_argv()),
            Backend::Docker { image, tty } => {
                let identity = docker::caller_identity().map_err(ConfinementError::Identity)?;
                Ok(docker::argv(
                    &self.layout,
                    &self.options,
                    image,
                    *tty,
                    identity,
                    &self.command,
                ))
            }
        }
    }

    /// The bubblewrap list, numbered as in the story that specified it.
    fn bubblewrap_argv(&self) -> Vec<OsString> {
        let mut argv: Vec<OsString> = Vec::new();
        let mut push =
            |arguments: &[&OsStr]| argv.extend(arguments.iter().map(|a| (*a).to_owned()));

        // 1. Namespaces and lifetime.
        push(&[o("--unshare-user"), o("--disable-userns")]);
        push(&[o("--unshare-ipc"), o("--unshare-pid"), o("--unshare-uts")]);
        if !self.options.network {
            push(&[o("--unshare-net")]);
        }
        push(&[o("--die-with-parent")]);

        // 2. Base system read-only, private proc/dev/tmp.
        push(&[o("--ro-bind"), o("/usr"), o("/usr")]);
        for base in ["/bin", "/lib", "/lib64"] {
            push(&[o("--ro-bind-try"), o(base), o(base)]);
        }
        push(&[
            o("--proc"),
            o("/proc"),
            o("--dev"),
            o("/dev"),
            o("--tmpfs"),
            o("/tmp"),
        ]);

        // 3. Declared read-only roots, at their own path.
        for root in &self.options.read_only_roots {
            push(&[o("--ro-bind"), root.as_os_str(), root.as_os_str()]);
        }

        // 4. The mirrored workspace: tmpfs, one writable bind per directory, parents read-only.
        //
        // `--remount-ro` acts on the mount at that exact path and on nothing under it, so it makes
        // the synthesized parents read-only while the binds beneath them stay writable. When a
        // mapping *is* `/workspace` — the default layout, with no `--dir` — that same argument
        // would remount the writable bind itself and leave the caller nothing to write to; there
        // are no synthesized parents to protect in that case, so it is omitted.
        let root_is_bound = self.layout.binds_workspace_root();
        push(&[o("--tmpfs"), o(WORKSPACE)]);
        for mapping in &self.layout.mappings {
            push(&[
                o("--bind"),
                mapping.host.as_os_str(),
                mapping.mount.as_os_str(),
            ]);
        }
        if !root_is_bound {
            push(&[o("--remount-ro"), o(WORKSPACE)]);
        }

        // 5. Working directory and the shaped environment.
        push(&[o("--chdir"), self.layout.cwd.as_os_str(), o("--clearenv")]);
        for (name, value) in [
            ("PATH", "/usr/bin:/bin"),
            ("HOME", "/tmp"),
            ("LANG", "C.UTF-8"),
            ("LC_ALL", "C.UTF-8"),
            ("TZ", "UTC"),
        ] {
            push(&[o("--setenv"), o(name), o(value)]);
        }
        if let Some(term) = &self.options.term {
            push(&[o("--setenv"), o("TERM"), term]);
        }
        for (name, value) in &self.options.env {
            push(&[o("--setenv"), name, value]);
        }

        // 6. The command.
        push(&[o("--")]);
        let command = self
            .command
            .iter()
            .map(OsString::as_os_str)
            .collect::<Vec<_>>();
        push(&command);
        argv
    }

    /// A `Command` that would run this confinement, for a caller that wants to wait rather than
    /// exec.
    ///
    /// # Errors
    ///
    /// As [`Confinement::argv`].
    pub fn command(&self) -> Result<Command, ConfinementError> {
        let mut command = Command::new(self.program());
        command.args(self.argv()?);
        Ok(command)
    }

    /// Check the terminal hazard (bubblewrap only: Docker allocates its own pseudo-terminal), then
    /// replace the current process with the backend's program.
    ///
    /// # Errors
    ///
    /// `legacy_tiocsti` is `1` and not allowed; the program is missing; or `exec` failed.
    pub fn run(self) -> Result<Infallible, ConfinementError> {
        if self.options.backend == Backend::Bubblewrap
            && !self.options.allow_tiocsti
            && legacy_tiocsti_enabled(Path::new(LEGACY_TIOCSTI_SYSCTL))
        {
            return Err(ConfinementError::LegacyTiocsti);
        }
        let error = self.command()?.exec();
        let program = self.program().to_path_buf();
        if error.kind() == io::ErrorKind::NotFound {
            return Err(ConfinementError::ProgramMissing(program));
        }
        Err(ConfinementError::Exec(program, error))
    }
}

/// `true` only when the sysctl exists and reads `1`. An absent sysctl is a kernel before 6.2,
/// where `TIOCSTI` is unconditionally available and the hazard is real; that reads as enabled too.
fn legacy_tiocsti_enabled(sysctl: &Path) -> bool {
    match std::fs::read_to_string(sysctl) {
        Ok(value) => value.trim() != "0",
        Err(_) => true,
    }
}

fn o(s: &str) -> &OsStr {
    OsStr::new(s)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use super::{Backend, Confinement, ConfinementError, Options, legacy_tiocsti_enabled};
    use crate::layout::{Layout, Mapping};

    fn two_directory_layout() -> Layout {
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

    #[test]
    fn argv_for_a_two_directory_layout_is_exact() {
        let options = Options {
            term: Some(OsString::from("xterm")),
            ..Options::default()
        };
        let confinement =
            Confinement::new(two_directory_layout(), options, ["/bin/sh", "-l"]).expect("valid");
        let argv = confinement
            .argv()
            .unwrap()
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let expected: Vec<&str> = vec![
            "--unshare-user",
            "--disable-userns",
            "--unshare-ipc",
            "--unshare-pid",
            "--unshare-uts",
            "--unshare-net",
            "--die-with-parent",
            "--ro-bind",
            "/usr",
            "/usr",
            "--ro-bind-try",
            "/bin",
            "/bin",
            "--ro-bind-try",
            "/lib",
            "/lib",
            "--ro-bind-try",
            "/lib64",
            "/lib64",
            "--proc",
            "/proc",
            "--dev",
            "/dev",
            "--tmpfs",
            "/tmp",
            "--tmpfs",
            "/workspace",
            "--bind",
            "/home/me/work/app",
            "/workspace/app",
            "--bind",
            "/home/me/work/lib",
            "/workspace/lib",
            "--remount-ro",
            "/workspace",
            "--chdir",
            "/workspace/app",
            "--clearenv",
            "--setenv",
            "PATH",
            "/usr/bin:/bin",
            "--setenv",
            "HOME",
            "/tmp",
            "--setenv",
            "LANG",
            "C.UTF-8",
            "--setenv",
            "LC_ALL",
            "C.UTF-8",
            "--setenv",
            "TZ",
            "UTC",
            "--setenv",
            "TERM",
            "xterm",
            "--",
            "/bin/sh",
            "-l",
        ];
        assert_eq!(argv, expected);
    }

    /// The default layout exactly as [`Layout::plan`] computes it: no `--dir`, so the working
    /// directory is the ancestor and is bound at `/workspace` itself.
    fn default_layout() -> (tempfile::TempDir, Layout) {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = std::fs::canonicalize(dir.path()).expect("canonical tempdir");
        let layout = Layout::plan(&root, &[]).expect("layout");
        (dir, layout)
    }

    #[test]
    fn the_default_layout_binds_the_workspace_writable_and_is_not_remounted() {
        let (_keep, layout) = default_layout();
        let host = layout.mappings[0].host.clone();
        let confinement = Confinement::new(layout, Options::default(), ["/bin/sh"]).expect("valid");
        let argv = confinement.argv().unwrap();
        let bind = argv.iter().position(|a| a == "--bind").expect("one bind");
        assert_eq!(Path::new(&argv[bind + 1]), host);
        assert_eq!(Path::new(&argv[bind + 2]), Path::new("/workspace"));
        assert!(
            !argv.iter().any(|a| a == "--remount-ro"),
            "remounting /workspace read-only would take the only writable bind away: {argv:?}"
        );
    }

    #[test]
    fn a_mirrored_layout_still_remounts_the_synthesized_root_read_only() {
        let confinement =
            Confinement::new(two_directory_layout(), Options::default(), ["/bin/sh"]).unwrap();
        let argv = confinement.argv().unwrap();
        let remount = argv
            .iter()
            .position(|a| a == "--remount-ro")
            .expect("present");
        assert_eq!(argv[remount + 1], "/workspace");
        let last_bind = argv
            .iter()
            .rposition(|a| a == "/workspace/lib")
            .expect("present");
        assert!(
            last_bind < remount,
            "the remount must follow every bind, or it would not cover the parents"
        );
    }

    #[test]
    fn network_drops_only_unshare_net_and_roots_precede_the_workspace() {
        let options = Options {
            network: true,
            read_only_roots: vec![PathBuf::from("/usr/share")],
            ..Options::default()
        };
        let confinement = Confinement::new(two_directory_layout(), options, ["/bin/sh"]).unwrap();
        let argv = confinement.argv().unwrap();
        assert!(!argv.iter().any(|a| a == "--unshare-net"));
        let root = argv.iter().position(|a| a == "/usr/share").unwrap();
        let workspace = argv.iter().position(|a| a == "/workspace").unwrap();
        assert!(root < workspace, "root at {root}, workspace at {workspace}");
    }

    #[test]
    fn refuses_an_empty_command_and_a_bad_root() {
        let empty: [&str; 0] = [];
        assert!(matches!(
            Confinement::new(two_directory_layout(), Options::default(), empty),
            Err(ConfinementError::EmptyCommand)
        ));
        let options = Options {
            read_only_roots: vec![PathBuf::from("relative")],
            ..Options::default()
        };
        assert!(matches!(
            Confinement::new(two_directory_layout(), options, ["/bin/sh"]),
            Err(ConfinementError::ReadOnlyRoot(_))
        ));
    }

    #[test]
    fn docker_backend_names_docker_and_its_own_argv() {
        let options = Options {
            backend: Backend::Docker {
                image: "debian:stable-slim".into(),
                tty: false,
            },
            ..Options::default()
        };
        let confinement = Confinement::new(two_directory_layout(), options, ["/bin/sh"]).unwrap();
        assert_eq!(confinement.program(), std::path::Path::new("docker"));
        let argv = confinement.argv().unwrap();
        assert_eq!(argv[0], "run");
        assert!(argv.iter().any(|a| a == "debian:stable-slim"));
        assert!(!argv.iter().any(|a| a == "--unshare-user"));
    }

    #[test]
    fn missing_sysctl_reads_as_hazard_present() {
        let dir = tempfile::tempdir().unwrap();
        assert!(legacy_tiocsti_enabled(&dir.path().join("absent")));
        let zero = dir.path().join("zero");
        std::fs::write(&zero, "0\n").unwrap();
        assert!(!legacy_tiocsti_enabled(&zero));
        let one = dir.path().join("one");
        std::fs::write(&one, "1\n").unwrap();
        assert!(legacy_tiocsti_enabled(&one));
    }
}
