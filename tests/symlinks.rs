#![cfg(unix)]
use rtree::{Cut, NodeKind, SortBy, Tree, TreeOptions};
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
    assert_eq!(link.cut, None);
}

#[test]
fn self_referential_link_terminates() {
    let dir = tempdir().unwrap();
    symlink(".", dir.path().join("self")).unwrap();

    let tree = followed(dir.path());
    let link = tree
        .root
        .children
        .iter()
        .find(|n| n.name == "self")
        .unwrap();
    assert_eq!(link.cut, Some(Cut::Cycle));
    assert!(tree.warnings.iter().any(|w| w.contains("cycle")));
}

#[test]
fn two_links_to_the_same_directory_are_both_traversed() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("real")).unwrap();
    fs::write(dir.path().join("real/inside"), "x").unwrap();
    symlink("real", dir.path().join("a")).unwrap();
    symlink("real", dir.path().join("b")).unwrap();

    let tree = followed(dir.path());
    for name in ["a", "b"] {
        let link = tree.root.children.iter().find(|n| n.name == name).unwrap();
        assert_eq!(link.children.len(), 1, "{name} must be traversed");
        assert_eq!(link.cut, None);
    }
    assert!(tree.warnings.is_empty(), "false cycle: {:?}", tree.warnings);
}

#[test]
fn link_back_to_root_is_cut_with_cycle_warning() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    symlink("..", dir.path().join("sub/up")).unwrap();

    let tree = followed(dir.path());
    let sub = tree.root.children.iter().find(|n| n.name == "sub").unwrap();
    let up = sub.children.iter().find(|n| n.name == "up").unwrap();
    assert_eq!(up.cut, Some(Cut::Cycle));
    assert!(up.children.is_empty());
    assert_eq!(tree.warnings.len(), 1);
    assert!(tree.warnings[0].contains("links back to .."));
    assert_eq!(tree.root.children.len(), 1);
}

#[test]
fn link_to_directory_seen_in_another_branch_is_not_a_cycle() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("a/shared")).unwrap();
    fs::create_dir(dir.path().join("b")).unwrap();
    symlink("../a/shared", dir.path().join("b/shared")).unwrap();

    let tree = followed(dir.path());
    let b = tree.root.children.iter().find(|n| n.name == "b").unwrap();
    let shared = b.children.iter().find(|n| n.name == "shared").unwrap();
    assert_eq!(shared.cut, None);
    assert!(tree.warnings.is_empty(), "false cycle: {:?}", tree.warnings);
}

#[test]
fn follow_depth_limit_cuts_nested_links() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("real/inner")).unwrap();
    symlink("real", dir.path().join("one")).unwrap();
    symlink("inner", dir.path().join("real/two")).unwrap();

    let tree = Tree::build(
        dir.path().to_path_buf(),
        TreeOptions {
            follow_links: true,
            max_follow_depth: Some(1),
            sort_by: SortBy::Name,
            ..Default::default()
        },
    )
    .unwrap();
    let one = tree.root.children.iter().find(|n| n.name == "one").unwrap();
    assert_eq!(one.cut, None);
    let two = one.children.iter().find(|n| n.name == "two").unwrap();
    assert_eq!(two.cut, Some(Cut::DepthLimit));
    assert!(
        tree.warnings
            .iter()
            .any(|w| w.contains("follow depth limit"))
    );
}
