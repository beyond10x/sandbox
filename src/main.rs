//! `b10x-sandbox`: respawn the current shell inside a bubblewrap sandbox.
//!
//! Clap over the library and nothing else. The one convenience this file adds is resolving a
//! relative `--dir` or `--ro` against the caller's working directory, lexically, so `../lib` is
//! usable from a shell; a symlink in the result is still refused by the library.

use std::ffi::OsString;
use std::io::{IsTerminal as _, Write as _};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, ValueEnum};

use b10x_sandbox::{Backend, Confinement, Layout, Options};

/// Respawn the current shell inside a bubblewrap sandbox.
///
/// The working directory is bound writable at /workspace. With --dir, the common ancestor of the
/// working directory and every --dir becomes /workspace and each directory is bound at its path
/// relative to that ancestor; nothing in between is visible. No network unless --net.
#[derive(Debug, Parser)]
#[command(name = "b10x-sandbox", version, about, long_about)]
struct Cli {
    /// A host directory to bind writable, mirrored under /workspace. Repeatable.
    #[arg(long = "dir", value_name = "PATH")]
    dirs: Vec<PathBuf>,

    /// A host directory to bind read-only at its own path, such as a toolchain home. Repeatable.
    #[arg(long = "ro", value_name = "PATH")]
    read_only: Vec<PathBuf>,

    /// Keep the host network namespace.
    #[arg(long)]
    net: bool,

    /// Which program confines the process.
    #[arg(long, value_enum, default_value_t = BackendChoice::Bubblewrap)]
    backend: BackendChoice,

    /// The image for --backend docker.
    #[arg(long, value_name = "NAME", default_value = b10x_sandbox::docker::DEFAULT_IMAGE)]
    image: String,

    /// Run even when `dev.tty.legacy_tiocsti` is 1.
    #[arg(long)]
    allow_tiocsti: bool,

    /// Print the bwrap argv, one argument per line, and exit without running it.
    #[arg(long)]
    dry_run: bool,

    /// The command to run. Default: $SHELL (or /bin/sh when unset) under bubblewrap, /bin/sh
    /// under docker, where the image decides which shells exist.
    #[arg(value_name = "CMD", trailing_var_arg = true)]
    command: Vec<OsString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum BackendChoice {
    Bubblewrap,
    Docker,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("b10x-sandbox: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("working directory: {e}"))?;
    let dirs = cli
        .dirs
        .iter()
        .map(|dir| absolutise(&cwd, dir))
        .collect::<Vec<_>>();
    let layout = Layout::plan(&cwd, &dirs).map_err(|e| e.to_string())?;

    let backend = match cli.backend {
        BackendChoice::Bubblewrap => Backend::Bubblewrap,
        BackendChoice::Docker => Backend::Docker {
            image: cli.image,
            tty: std::io::stdin().is_terminal(),
        },
    };
    let command = if !cli.command.is_empty() {
        cli.command
    } else if backend == Backend::Bubblewrap {
        vec![std::env::var_os("SHELL").unwrap_or_else(|| OsString::from("/bin/sh"))]
    } else {
        vec![OsString::from("/bin/sh")]
    };
    let options = Options {
        backend,
        network: cli.net,
        allow_tiocsti: cli.allow_tiocsti,
        read_only_roots: cli
            .read_only
            .iter()
            .map(|root| absolutise(&cwd, root))
            .collect(),
        term: std::env::var_os("TERM"),
        ..Options::default()
    };
    let confinement = Confinement::new(layout, options, command).map_err(|e| e.to_string())?;

    if cli.dry_run {
        let argv = confinement.argv().map_err(|e| e.to_string())?;
        let mut out = std::io::stdout().lock();
        let written = writeln!(out, "{}", confinement.program().display()).and_then(|()| {
            argv.iter()
                .try_for_each(|argument| writeln!(out, "{}", argument.to_string_lossy()))
        });
        return match written {
            Ok(()) => Ok(ExitCode::SUCCESS),
            // A reader that stopped early (`| head`) is not an error of ours.
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(ExitCode::SUCCESS),
            Err(e) => Err(format!("stdout: {e}")),
        };
    }
    match confinement.run() {
        Ok(never) => match never {},
        Err(error) => Err(error.to_string()),
    }
}

/// Join a relative path onto `cwd` and resolve `.` and `..` lexically. Symlinks are left alone,
/// so the library's canonical check still refuses them.
fn absolutise(cwd: &Path, path: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    let mut out = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}
