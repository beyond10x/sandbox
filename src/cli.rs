//! The clap definition of the `b10x-sandbox` command line.
//!
//! It lives in the library so that `sandbox-docs` can walk it to generate the CLI reference. The
//! binary (`src/main.rs`) parses it and turns it into a [`crate::Confinement`].

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};

/// Respawn the current shell inside a bubblewrap sandbox.
///
/// The working directory is bound writable at /workspace. With --dir, the common ancestor of the
/// working directory and every --dir becomes /workspace and each directory is bound at its path
/// relative to that ancestor; nothing in between is visible. No network unless --net.
#[derive(Debug, Parser)]
#[command(name = "b10x-sandbox", version, about, long_about)]
pub struct Cli {
    /// A host directory to bind writable, mirrored under /workspace. Repeatable.
    #[arg(long = "dir", value_name = "PATH")]
    pub dirs: Vec<PathBuf>,

    /// A host directory to bind read-only at its own path, such as a toolchain home. Repeatable.
    #[arg(long = "ro", value_name = "PATH")]
    pub read_only: Vec<PathBuf>,

    /// Keep the host network namespace.
    #[arg(long)]
    pub net: bool,

    /// Which program confines the process.
    #[arg(long, value_enum, default_value_t = BackendChoice::Bubblewrap)]
    pub backend: BackendChoice,

    /// The image for --backend docker.
    #[arg(long, value_name = "NAME", default_value = crate::docker::DEFAULT_IMAGE)]
    pub image: String,

    /// Run even when `dev.tty.legacy_tiocsti` is 1.
    #[arg(long)]
    pub allow_tiocsti: bool,

    /// Print the program and its argv, one argument per line, and exit without running it.
    #[arg(long)]
    pub dry_run: bool,

    /// The command to run. Default: $SHELL (or /bin/sh when unset) under bubblewrap, /bin/sh
    /// under docker, where the image decides which shells exist.
    #[arg(value_name = "CMD", trailing_var_arg = true)]
    pub command: Vec<OsString>,
}

/// The `--backend` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BackendChoice {
    // Variant doc comments would change the possible-values text of `--help`.
    Bubblewrap,
    Docker,
}
