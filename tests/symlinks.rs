#![cfg(unix)]
use rtree::{NodeKind, SortBy, Tree, TreeOptions};
use std::fs;
use std::os::unix::fs::symlink;
use tempfile::tempdir;

fn followed(root: &std::path::Path) -> Tree {
    Tree::build(
        root.to_path_buf(),
        TreeOptions {
            follow_links: true,
            sort_by: SortBy::Name,
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn file_symlink_keeps_marker_under_follow() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("real.txt"), "x").unwrap();
    symlink("real.txt", dir.path().join("link.txt")).unwrap();

    let tree = followed(dir.path());
    let link = tree
        .root
        .children
        .iter()
        .find(|n| n.name == "link.txt")
        .unwrap();
    assert!(link.is_symlink);
    assert_eq!(link.kind, NodeKind::File);
    assert_eq!(
        link.link_target.as_deref(),
        Some(std::path::Path::new("real.txt"))
    );
}

#[test]
fn dir_symlink_is_followed_but_still_marked() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("real")).unwrap();
    fs::write(dir.path().join("real/inside"), "x").unwrap();
    symlink("real", dir.path().join("link")).unwrap();

    let tree = followed(dir.path());
    let link = tree
        .root
        .children
        .iter()
        .find(|n| n.name == "link")
        .unwrap();
    assert!(link.is_symlink);
    assert_eq!(link.kind, NodeKind::Directory);
    assert_eq!(link.children.len(), 1);
}

#[test]
fn self_referential_link_terminates() {
    let dir = tempdir().unwrap();
    symlink(".", dir.path().join("self")).unwrap();

    let tree = followed(dir.path());
    assert!(tree.warnings.iter().any(|w| w.contains("symlink loop")));
}
