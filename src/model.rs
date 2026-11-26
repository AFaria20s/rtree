use colored::Color;

use std::{
    cmp::Ordering,
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
    pub is_symlink: bool,
    pub link_target: Option<PathBuf>,
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
    pub theme: Theme,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub directory: Color,
    pub symlink: Color,
    pub count: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            directory: Color::Blue,
            symlink: Color::Yellow,
            count: Color::Cyan,
        }
    }
}

impl Theme {
    pub fn from_env() -> Self {
        let mut theme = Self::default();
        if let Ok(spec) = std::env::var("RTREE_COLORS") {
            theme.apply(&spec);
        }
        theme
    }

    /// Parses specs like "dir=magenta,count=bright black"
    pub fn apply(&mut self, spec: &str) {
        for pair in spec.split(',') {
            let Some((key, value)) = pair.split_once('=') else {
                continue;
            };
            let Ok(color) = value.trim().parse::<Color>() else {
                continue;
            };
            match key.trim() {
                "dir" => self.directory = color,
                "link" => self.symlink = color,
                "count" => self.count = color,
                _ => {}
            }
        }
    }
}

#[test]
fn apply_overrides_only_given_colors() {
    let mut theme = Theme::default();
    theme.apply("dir=red, bogus=blue, count=nonsense");
    assert_eq!(theme.directory, Color::Red);
    assert_eq!(theme.count, Color::Cyan);
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
            theme: Theme::default(),
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
        if !path.exists() && fs::symlink_metadata(&path).is_err() {
            return Err(Error::MissingPath(path));
        }
        let mut warnings = Vec::new();
        let mut ancestors = Vec::new();
        let root = build_node(&path, &options, 0, &mut ancestors, &mut warnings);
        Ok(Self { root, warnings })
    }

    pub fn write_to<W: io::Write>(&self, writer: &mut W, options: &TreeOptions) -> io::Result<()> {
        render::write_tree(writer, self, options)
    }
}

fn entry_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn build_node(
    path: &Path,
    options: &TreeOptions,
    depth: usize,
    ancestors: &mut Vec<PathBuf>,
    warnings: &mut Vec<String>,
) -> Node {
    // lstat first: this is the only call that still tells the truth about the entry.
    let link_meta = fs::symlink_metadata(path);
    let is_symlink = matches!(&link_meta, Ok(m) if m.file_type().is_symlink());
    let link_target = if is_symlink {
        fs::read_link(path).ok()
    } else {
        None
    };

    // Resolve only when asked. A broken link under -l falls back to the lstat result.
    let meta = if options.follow_links {
        fs::metadata(path).or(link_meta)
    } else {
        link_meta
    };

    let mut node = Node {
        name: entry_name(path),
        path: path.to_path_buf(),
        kind: match &meta {
            Ok(m) if m.is_dir() => NodeKind::Directory,
            Ok(m) if m.is_file() => NodeKind::File,
            _ if is_symlink => NodeKind::Symlink,
            _ => NodeKind::Other,
        },
        child_count: 0,
        children: Vec::new(),
        modified: meta.ok().and_then(|m| m.modified().ok()),
        is_symlink,
        link_target,
    };

    if node.kind != NodeKind::Directory {
        return node;
    }

    // Count without descending when depth is exhausted.
    if !options.max_depth.is_none_or(|max| depth < max) {
        node.child_count = count_children(path, options, warnings);
        return node;
    }

    // Loop guard: a symlink to an ancestor would otherwise recurse forever.
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if ancestors.contains(&canonical) {
        warnings.push(format!("{}: symlink loop, not descending", path.display()));
        return node;
    }

    ancestors.push(canonical);
    let entries: Vec<PathBuf> = match fs::read_dir(path) {
        Ok(rd) => rd
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|entry| included(entry, options))
            .collect(),
        Err(error) => {
            warnings.push(format!("cannot read {}: {error}", path.display()));
            Vec::new()
        }
    };
    node.child_count = entries.len();
    for entry in &entries {
        node.children
            .push(build_node(entry, options, depth + 1, ancestors, warnings));
    }
    ancestors.pop();
    sort_children(&mut node.children, options);
    node
}

fn count_children(path: &Path, options: &TreeOptions, warnings: &mut Vec<String>) -> usize {
    match fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .filter(|entry| included(&entry.path(), options))
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

/// `include` plus the `--dirs-only` filter, resolving the entry when `-l` is on so a
/// symlink to a directory counts as a directory.
fn included(path: &Path, options: &TreeOptions) -> bool {
    if !include(path, options) {
        return false;
    }
    if !options.directories_only {
        return true;
    }
    let meta = if options.follow_links {
        fs::metadata(path)
    } else {
        fs::symlink_metadata(path)
    };
    meta.map(|m| m.is_dir()).unwrap_or(false)
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
