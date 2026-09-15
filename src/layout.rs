//! Where each host directory lands under `/workspace`.
//!
//! A pure computation over paths. The one rule: the longest common ancestor of the working
//! directory and every declared directory becomes `/workspace`, and each input is bound at its
//! path relative to that ancestor. Nothing between the ancestor and an input is visible, because
//! only the inputs are bound; the parents are synthesized as empty directories by bubblewrap.

use std::path::{Component, Path, PathBuf};

use thiserror::Error;

/// The mount point of the workspace inside the sandbox.
pub const WORKSPACE: &str = "/workspace";

/// One host directory and the absolute path it appears at inside the sandbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapping {
    /// The canonical host directory.
    pub host: PathBuf,
    /// Where it is bound inside the sandbox, at or under `/workspace`.
    pub mount: PathBuf,
}

/// The complete mirrored layout: every writable bind and where the shell starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// The host directory that became `/workspace`.
    pub ancestor: PathBuf,
    /// The working directory's mount inside the sandbox.
    pub cwd: PathBuf,
    /// Every bind, working directory included, in input order with the working directory first.
    pub mappings: Vec<Mapping>,
}

/// Why an input path was refused. Every variant names the path so the caller can say which.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LayoutError {
    #[error("{0}: not an absolute path")]
    NotAbsolute(PathBuf),
    #[error("{0}: not canonical (a symlink, `.` or `..` in a component)")]
    NotCanonical(PathBuf),
    #[error("{0}: not an existing directory")]
    NotADirectory(PathBuf),
    #[error("{0}: listed twice")]
    Duplicate(PathBuf),
    #[error("{inner}: nested inside {outer}, already writable through it")]
    Nested { inner: PathBuf, outer: PathBuf },
}

impl Layout {
    /// Compute the layout for `cwd` and the declared `dirs`.
    ///
    /// With no `dirs`, `cwd` maps to `/workspace`. Otherwise the longest common ancestor of all
    /// inputs maps to `/workspace` and each input to `/workspace/<relative>`; an input that *is*
    /// the ancestor maps to `/workspace` itself.
    ///
    /// # Errors
    ///
    /// Every input is validated before anything is computed; the first refusal is returned and
    /// nothing is adjusted.
    pub fn plan(cwd: &Path, dirs: &[PathBuf]) -> Result<Self, LayoutError> {
        let mut inputs = Vec::with_capacity(dirs.len() + 1);
        inputs.push(validate(cwd)?);
        for dir in dirs {
            let dir = validate(dir)?;
            if let Some(twin) = inputs.iter().find(|seen| **seen == dir) {
                return Err(LayoutError::Duplicate(twin.clone()));
            }
            inputs.push(dir);
        }
        for (index, inner) in inputs.iter().enumerate() {
            if let Some(outer) = inputs
                .iter()
                .enumerate()
                .find(|(other, outer)| *other != index && inner.starts_with(outer))
                .map(|(_, outer)| outer)
            {
                return Err(LayoutError::Nested {
                    inner: inner.clone(),
                    outer: outer.clone(),
                });
            }
        }

        let ancestor = if dirs.is_empty() {
            inputs[0].clone()
        } else {
            common_ancestor(&inputs)
        };

        let mappings = inputs
            .iter()
            .map(|host| Mapping {
                host: host.clone(),
                mount: mount_for(&ancestor, host),
            })
            .collect::<Vec<_>>();
        let cwd = mappings[0].mount.clone();
        Ok(Self {
            ancestor,
            cwd,
            mappings,
        })
    }
}

/// Refuse a path that is not absolute, not canonical, or not an existing directory.
///
/// # Errors
///
/// Named after the first property the path fails.
pub fn canonical_directory(path: &Path) -> Result<PathBuf, LayoutError> {
    validate(path)
}

fn validate(path: &Path) -> Result<PathBuf, LayoutError> {
    if !path.is_absolute() {
        return Err(LayoutError::NotAbsolute(path.to_path_buf()));
    }
    if path
        .components()
        .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(LayoutError::NotCanonical(path.to_path_buf()));
    }
    let canonical =
        std::fs::canonicalize(path).map_err(|_| LayoutError::NotADirectory(path.to_path_buf()))?;
    if canonical != path {
        return Err(LayoutError::NotCanonical(path.to_path_buf()));
    }
    if !canonical.is_dir() {
        return Err(LayoutError::NotADirectory(path.to_path_buf()));
    }
    Ok(canonical)
}

/// The longest path every input starts with. At least `/`, since every input is absolute.
fn common_ancestor(inputs: &[PathBuf]) -> PathBuf {
    let mut ancestor = inputs[0].clone();
    for input in &inputs[1..] {
        while !input.starts_with(&ancestor) {
            if !ancestor.pop() {
                break;
            }
        }
    }
    if ancestor.as_os_str().is_empty() {
        PathBuf::from("/")
    } else {
        ancestor
    }
}

fn mount_for(ancestor: &Path, host: &Path) -> PathBuf {
    let relative = host
        .strip_prefix(ancestor)
        .expect("every input starts with the ancestor");
    Path::new(WORKSPACE).join(relative)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::{Layout, LayoutError, Mapping};

    /// A canonical temporary tree; `TMPDIR` may itself be a symlink on some hosts.
    fn tree() -> (TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = fs::canonicalize(dir.path()).expect("canonical tempdir");
        (dir, root)
    }

    #[test]
    fn no_dirs_maps_cwd_to_workspace() {
        let (_keep, root) = tree();
        let layout = Layout::plan(&root, &[]).expect("layout");
        assert_eq!(layout.ancestor, root);
        assert_eq!(layout.cwd, PathBuf::from("/workspace"));
        assert_eq!(
            layout.mappings,
            vec![Mapping {
                host: root,
                mount: PathBuf::from("/workspace")
            }]
        );
    }

    #[test]
    fn two_siblings_mirror_under_their_parent() {
        let (_keep, root) = tree();
        let a = root.join("x/a");
        let b = root.join("y/z/b");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        let layout = Layout::plan(&a, std::slice::from_ref(&b)).expect("layout");
        assert_eq!(layout.ancestor, root);
        assert_eq!(layout.cwd, PathBuf::from("/workspace/x/a"));
        assert_eq!(
            layout.mappings,
            vec![
                Mapping {
                    host: a,
                    mount: PathBuf::from("/workspace/x/a")
                },
                Mapping {
                    host: b,
                    mount: PathBuf::from("/workspace/y/z/b")
                },
            ]
        );
    }

    #[test]
    fn an_input_that_is_the_ancestor_maps_to_workspace_itself() {
        let (_keep, root) = tree();
        let a = root.join("a");
        fs::create_dir_all(&a).unwrap();
        // cwd is the ancestor of the declared dir: refused as nested, since `a` is already
        // writable through cwd.
        let nested = Layout::plan(&root, std::slice::from_ref(&a)).unwrap_err();
        assert_eq!(
            nested,
            LayoutError::Nested {
                inner: a.clone(),
                outer: root.clone()
            }
        );
        // A declared dir that is the ancestor of cwd is the same refusal the other way round.
        let nested = Layout::plan(&a, std::slice::from_ref(&root)).unwrap_err();
        assert_eq!(
            nested,
            LayoutError::Nested {
                inner: a,
                outer: root
            }
        );
    }

    #[test]
    fn refuses_a_relative_path() {
        let (_keep, root) = tree();
        let err = Layout::plan(&root, &[PathBuf::from("relative")]).unwrap_err();
        assert_eq!(err, LayoutError::NotAbsolute(PathBuf::from("relative")));
    }

    #[test]
    fn refuses_a_non_canonical_path() {
        let (_keep, root) = tree();
        let dotted = root.join("a/../b");
        let err = Layout::plan(&root, std::slice::from_ref(&dotted)).unwrap_err();
        assert_eq!(err, LayoutError::NotCanonical(dotted));
        let target = root.join("target");
        fs::create_dir_all(&target).unwrap();
        let link = root.join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let err = Layout::plan(&target, std::slice::from_ref(&link)).unwrap_err();
        assert_eq!(err, LayoutError::NotCanonical(link));
    }

    #[test]
    fn refuses_a_missing_directory_and_a_file() {
        let (_keep, root) = tree();
        let missing = root.join("missing");
        let err = Layout::plan(&root, std::slice::from_ref(&missing)).unwrap_err();
        assert_eq!(err, LayoutError::NotADirectory(missing));
        let file = root.join("file");
        fs::write(&file, b"").unwrap();
        let err = Layout::plan(&root, std::slice::from_ref(&file)).unwrap_err();
        assert_eq!(err, LayoutError::NotADirectory(file));
    }

    #[test]
    fn refuses_a_duplicate() {
        let (_keep, root) = tree();
        let a = root.join("a");
        fs::create_dir_all(&a).unwrap();
        let err = Layout::plan(&root.join("a"), std::slice::from_ref(&a)).unwrap_err();
        assert_eq!(err, LayoutError::Duplicate(a));
    }

    #[test]
    fn inputs_under_different_top_level_directories_mirror_from_root() {
        let (_keep, root) = tree();
        let a = root.join("a");
        fs::create_dir_all(&a).unwrap();
        let layout = Layout::plan(&a, &[PathBuf::from("/usr")]).expect("layout");
        assert_eq!(layout.ancestor, PathBuf::from("/"));
        assert_eq!(
            layout.cwd,
            PathBuf::from("/workspace").join(a.strip_prefix("/").unwrap())
        );
        assert_eq!(layout.mappings[1].mount, PathBuf::from("/workspace/usr"));
    }
}
