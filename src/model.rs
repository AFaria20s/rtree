use std::{
    cmp::Ordering,
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{error::Error, render};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub path: PathBuf,
    pub kind: NodeKind,
    pub child_count: usize,
    pub children: Vec<Node>,
    pub(crate) modified: Option<SystemTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Name,
    Modified,
    None,
}

#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub path: String,
    pub max_depth: Option<usize>,
    pub show_hidden: bool,
    pub show_counts: bool,
    pub directories_only: bool,
    pub follow_links: bool,
    pub ignored: Vec<String>,
    pub sort_by: SortBy,
    pub reverse: bool,
    pub ascii: bool,
    pub show_summary: bool,
    pub quiet: bool,
    pub no_color: bool,
}

impl Default for TreeOptions {
    fn default() -> Self {
        Self {
            path: ".".into(),
            max_depth: None,
            show_hidden: false,
            show_counts: true,
            directories_only: false,
            follow_links: false,
            ignored: vec![],
            sort_by: SortBy::Name,
            reverse: false,
            ascii: false,
            show_summary: false,
            quiet: false,
            no_color: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tree {
    pub root: Node,
    pub warnings: Vec<String>,
}

impl Tree {
    pub fn build(path: PathBuf, options: TreeOptions) -> Result<Self, Error> {
        if !path.exists() {
            return Err(Error::MissingPath(path));
        }
        let mut warnings = Vec::new();
        let mut visited = HashSet::new();
        let root = build_node(&path, 0, &options, &mut warnings, &mut visited)?;
        Ok(Self { root, warnings })
    }

    pub fn write_to<W: io::Write>(&self, writer: &mut W, options: &TreeOptions) -> io::Result<()> {
        render::write_tree(writer, self, options)
    }
}

fn build_node(
    path: &Path,
    depth: usize,
    options: &TreeOptions,
    warnings: &mut Vec<String>,
    visited: &mut HashSet<PathBuf>,
) -> Result<Node, Error> {
    let metadata = if options.follow_links {
        fs::metadata(path)
    } else {
        fs::symlink_metadata(path)
    }
    .map_err(|source| Error::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let kind = if metadata.file_type().is_symlink() {
        NodeKind::Symlink
    } else if metadata.is_dir() {
        NodeKind::Directory
    } else if metadata.is_file() {
        NodeKind::File
    } else {
        NodeKind::Other
    };
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let mut children = Vec::new();
    let cycle = options.follow_links
        && kind == NodeKind::Directory
        && fs::canonicalize(path)
            .ok()
            .is_some_and(|canonical| !visited.insert(canonical));
    if cycle {
        warnings.push(format!("skipping cyclic link at {}", path.display()));
    }
    let can_descend = kind == NodeKind::Directory
        && !cycle
        && options.max_depth.is_none_or(|limit| depth < limit);
    if can_descend {
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) if include(&entry.path(), options) => {
                            match build_node(&entry.path(), depth + 1, options, warnings, visited) {
                                Ok(child)
                                    if !options.directories_only
                                        || child.kind == NodeKind::Directory =>
                                {
                                    children.push(child)
                                }
                                Ok(_) => {}
                                Err(error) => warnings.push(error.to_string()),
                            }
                        }
                        Ok(_) => {}
                        Err(error) => warnings
                            .push(format!("cannot read entry in {}: {error}", path.display())),
                    }
                }
            }
            Err(error) => warnings.push(format!("cannot read {}: {error}", path.display())),
        }
    }
    sort_children(&mut children, options);
    let child_count = if options.show_counts && kind == NodeKind::Directory && !can_descend {
        count_children(path, options, warnings)
    } else {
        children.len()
    };
    Ok(Node {
        name,
        path: path.to_path_buf(),
        kind,
        child_count,
        children,
        modified: metadata.modified().ok(),
    })
}

fn count_children(path: &Path, options: &TreeOptions, warnings: &mut Vec<String>) -> usize {
    match fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .filter(|entry| {
                include(&entry.path(), options)
                    && (!options.directories_only
                        || entry.file_type().is_ok_and(|kind| kind.is_dir()))
            })
            .count(),
        Err(error) => {
            warnings.push(format!("cannot read {}: {error}", path.display()));
            0
        }
    }
}

fn include(path: &Path, options: &TreeOptions) -> bool {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    (options.show_hidden || !name.starts_with('.'))
        && !options.ignored.iter().any(|pattern| name.contains(pattern))
}

fn sort_children(children: &mut [Node], options: &TreeOptions) {
    if options.sort_by == SortBy::None {
        return;
    }
    children.sort_by(|a, b| {
        let order = match options.sort_by {
            SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortBy::Modified => a.modified.cmp(&b.modified),
            SortBy::None => Ordering::Equal,
        };
        if options.reverse {
            order.reverse()
        } else {
            order
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn hidden_and_ignored_entries_are_not_counted() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join("visible")).unwrap();
        fs::create_dir(dir.path().join(".hidden")).unwrap();
        fs::write(dir.path().join("skip.log"), "x").unwrap();
        let tree = Tree::build(
            dir.path().to_path_buf(),
            TreeOptions {
                ignored: vec!["log".into()],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(tree.root.child_count, 1);
        assert_eq!(tree.root.children[0].name, "visible");
    }
    #[test]
    fn depth_zero_does_not_read_children() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("file"), "x").unwrap();
        let tree = Tree::build(
            dir.path().to_path_buf(),
            TreeOptions {
                max_depth: Some(0),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(tree.root.children.is_empty());
        assert_eq!(tree.root.child_count, 1);
    }
    #[test]
    fn directories_only_filters_files() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join("folder")).unwrap();
        fs::write(dir.path().join("file"), "x").unwrap();
        let tree = Tree::build(
            dir.path().to_path_buf(),
            TreeOptions {
                directories_only: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(tree.root.children.len(), 1);
        assert_eq!(tree.root.children[0].kind, NodeKind::Directory);
    }

    #[test]
    fn all_option_includes_hidden_entries() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".env"), "x").unwrap();
        let tree = Tree::build(
            dir.path().to_path_buf(),
            TreeOptions {
                show_hidden: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(tree.root.children[0].name, ".env");
    }

    #[test]
    fn reverse_name_sort_is_applied() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("alpha"), "x").unwrap();
        fs::write(dir.path().join("zulu"), "x").unwrap();
        let tree = Tree::build(
            dir.path().to_path_buf(),
            TreeOptions {
                reverse: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            tree.root
                .children
                .iter()
                .map(|node| node.name.as_str())
                .collect::<Vec<_>>(),
            vec!["zulu", "alpha"]
        );
    }

    #[test]
    fn a_file_can_be_used_as_the_root() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("single.txt");
        fs::write(&file, "x").unwrap();
        let tree = Tree::build(file, TreeOptions::default()).unwrap();
        assert_eq!(tree.root.kind, NodeKind::File);
        assert!(tree.root.children.is_empty());
    }
}
