//! The status data, `website/data/status.json` (`b10x-status/1`), from [`CAPABILITIES`].
//!
//! A shipped capability names the test that holds it, as `file::function`. Generation fails when
//! that file has no such function, so a claim cannot outlive the test it rests on. A capability
//! that is not shipped names none.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail, ensure};
use serde_json::json;

/// Where the status data lands, relative to the repository root.
pub const DATA: &str = "website/data/status.json";

/// The date the list below was last checked against `main`.
const AS_OF: &str = "2026-10-05";

/// One capability on the status page.
pub struct Capability {
    pub area: &'static str,
    pub label: &'static str,
    pub detail: &'static str,
    /// `shipped`, `decided` or `planned`.
    pub status: &'static str,
    /// For a shipped capability, the test that holds it: `path::function`.
    pub evidence: Option<&'static str>,
    pub href: &'static str,
}

/// Every capability the site claims, in the order the status table lists them.
pub const CAPABILITIES: &[Capability] = &[
    Capability {
        area: "Confinement",
        label: "Substrate's namespace and mount set under bubblewrap",
        detail: "User, ipc, pid, uts and net namespaces unshared, no nested user namespace, /usr /bin /lib /lib64 read-only, private /proc /dev /tmp, cleared environment, dies with its parent.",
        status: "shipped",
        evidence: Some(
            "tests/substrate_mirror.rs::the_bubblewrap_list_differs_from_substrates_only_where_its_module_doc_says",
        ),
        href: "/docs/concepts/confinement",
    },
    Capability {
        area: "Confinement",
        label: "No network unless asked",
        detail: "--net keeps the host network namespace and changes nothing else in the list.",
        status: "shipped",
        evidence: Some(
            "tests/substrate_mirror.rs::keeping_the_host_network_drops_unshare_net_and_nothing_else",
        ),
        href: "/docs/concepts/confinement",
    },
    Capability {
        area: "Confinement",
        label: "Undeclared host paths are absent",
        detail: "Observed under bwrap: a sibling of the working directory cannot be read, and a --ro root can be read but not written.",
        status: "shipped",
        evidence: Some(
            "tests/bubblewrap.rs::the_default_workspace_is_writable_and_the_undeclared_host_stays_out",
        ),
        href: "/docs/concepts/confinement",
    },
    Capability {
        area: "Confinement",
        label: "Terminal-injection guard",
        detail: "Refuses to run when dev.tty.legacy_tiocsti is 1, unless --allow-tiocsti; it stands in for bwrap --new-session.",
        status: "shipped",
        evidence: Some(
            "tests/substrate_mirror.rs::the_terminal_guard_that_stands_in_for_new_session_is_still_there",
        ),
        href: "/docs/concepts/confinement#the-terminal",
    },
    Capability {
        area: "Layout",
        label: "Mirrored /workspace with --dir",
        detail: "The common ancestor becomes /workspace; each directory is bound at its relative path, so ../lib resolves as on the host.",
        status: "shipped",
        evidence: Some(
            "tests/bubblewrap.rs::mirrored_directories_resolve_against_each_other_and_nothing_else_is_visible",
        ),
        href: "/docs/concepts/workspace-layout",
    },
    Capability {
        area: "Layout",
        label: "Input paths refused, never adjusted",
        detail: "Relative, non-canonical, missing, duplicated and nested inputs are each a named error.",
        status: "shipped",
        evidence: Some("src/layout.rs::refuses_a_non_canonical_path"),
        href: "/docs/concepts/workspace-layout#refusals",
    },
    Capability {
        area: "Backends",
        label: "Docker backend",
        detail: "--backend docker: the same layout through docker run as the caller's uid, no capabilities, no network, read-only image root.",
        status: "shipped",
        evidence: Some(
            "tests/docker.rs::mirrored_directories_resolve_against_each_other_and_nothing_else_is_visible",
        ),
        href: "/docs/guides/use-the-docker-backend",
    },
    Capability {
        area: "Backends",
        label: "A missing backend is a refusal, never a fallback",
        detail: "No bwrap or no docker exits 2 with the program's name; nothing runs unconfined or on the other backend.",
        status: "shipped",
        evidence: Some(
            "tests/backend_refusal.rs::a_missing_bubblewrap_never_falls_back_to_running_the_command_unconfined",
        ),
        href: "/docs/concepts/confinement#refusals",
    },
    Capability {
        area: "Interfaces",
        label: "b10x-sandbox command line",
        detail: "--dir, --ro, --net, --backend, --image, --dry-run, --allow-tiocsti and a trailing command.",
        status: "shipped",
        evidence: Some("tests/cli.rs::dry_run_prints_the_mirrored_argv_and_exits_zero"),
        href: "/docs/reference/cli",
    },
    Capability {
        area: "Interfaces",
        label: "b10x_sandbox Rust library",
        detail: "Layout::plan, Confinement::new, argv, command and run: the same computation the command line uses.",
        status: "shipped",
        evidence: Some("src/confinement.rs::argv_for_a_two_directory_layout_is_exact"),
        href: "/docs/guides/embed-the-library",
    },
];

/// Whether `source` defines a function called `name`.
fn defines(source: &str, name: &str) -> bool {
    source.contains(&format!("fn {name}("))
}

/// The status document for the repository at `root`.
pub fn document(root: &Path) -> Result<String> {
    document_for(root, CAPABILITIES)
}

fn document_for(root: &Path, capabilities: &[Capability]) -> Result<String> {
    let mut items = Vec::new();
    for capability in capabilities {
        ensure!(
            matches!(capability.status, "shipped" | "decided" | "planned"),
            "{}: status {} is not shipped, decided or planned",
            capability.label,
            capability.status
        );
        match (capability.status, capability.evidence) {
            ("shipped", Some(evidence)) => {
                let (file, function) = evidence
                    .split_once("::")
                    .with_context(|| format!("{evidence}: not `path::function`"))?;
                let source = fs::read_to_string(root.join(file))
                    .with_context(|| format!("{}: reading {file}", capability.label))?;
                ensure!(
                    defines(&source, function),
                    "{}: {file} has no test {function}",
                    capability.label
                );
            }
            ("shipped", None) => bail!("{}: shipped without a test", capability.label),
            (_, Some(_)) => bail!("{}: only a shipped item names a test", capability.label),
            (_, None) => {}
        }
        items.push(json!({
            "area": capability.area,
            "label": capability.label,
            "detail": capability.detail,
            "status": capability.status,
            "href": capability.href,
        }));
    }
    let document = json!({
        "format": "b10x-status/1",
        "asOf": AS_OF,
        "source": "sandbox-docs, from its capability list; every shipped item names the test that holds it",
        "items": items,
    });
    Ok(serde_json::to_string_pretty(&document)? + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> &'static Path {
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    fn capability(status: &'static str, evidence: Option<&'static str>) -> Capability {
        Capability {
            area: "A",
            label: "L",
            detail: "D",
            status,
            evidence,
            href: "/docs/",
        }
    }

    #[test]
    fn every_shipped_capability_rests_on_a_test_that_exists() {
        let document = document(root()).unwrap();
        assert!(document.contains("\"format\": \"b10x-status/1\""));
    }

    #[test]
    fn a_vanished_test_or_a_shipped_item_without_one_is_refused() {
        let gone = capability("shipped", Some("tests/cli.rs::no_such_test"));
        let error = document_for(root(), &[gone]).unwrap_err().to_string();
        assert!(error.contains("has no test no_such_test"), "{error}");
        assert!(document_for(root(), &[capability("shipped", None)]).is_err());
        assert!(
            document_for(
                root(),
                &[capability(
                    "planned",
                    Some("tests/cli.rs::dry_run_prints_the_mirrored_argv_and_exits_zero")
                )]
            )
            .is_err()
        );
        assert!(document_for(root(), &[capability("released", None)]).is_err());
        assert!(document_for(root(), &[capability("planned", None)]).is_ok());
    }
}
