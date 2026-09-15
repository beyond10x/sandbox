//! Respawn a process inside a bubblewrap sandbox that mirrors chosen host directories under
//! `/workspace`.
//!
//! Two modules, in the order a caller uses them:
//!
//! - [`layout`] computes *where* each host directory lands under `/workspace`. It is a pure
//!   function of paths and needs no bubblewrap.
//! - [`confinement`] turns a [`layout::Layout`] and some [`confinement::Options`] into the complete
//!   argv of the chosen [`confinement::Backend`] and runs it. Bubblewrap is the default;
//!   [`docker`] holds the `docker run` list.
//!
//! The isolation set is substrate's (`crates/substrate-host/src/process.rs` at substrate 0.7.0),
//! carried here as an argument list so it can be compared line by line. Linux and bubblewrap only.

pub mod confinement;
pub mod docker;
pub mod layout;

pub use confinement::{Backend, Confinement, ConfinementError, Options};
pub use layout::{Layout, LayoutError, Mapping};
