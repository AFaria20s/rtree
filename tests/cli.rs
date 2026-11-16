use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn default_output_shows_direct_child_counts() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("docs")).unwrap();
    fs::write(dir.path().join("docs").join("guide.md"), "hi").unwrap();
    fs::write(dir.path().join("main.rs"), "fn main() {} ").unwrap();
    Command::cargo_bin("rtree")
        .unwrap()
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[2]").and(predicate::str::contains("[1] docs/")));
}

#[test]
fn no_count_and_ascii_flags_change_output() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("child")).unwrap();
    Command::cargo_bin("rtree")
        .unwrap()
        .args(["--no-count", "--ascii"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("`- child/").and(predicate::str::contains("[0]").not()));
}

#[test]
fn missing_path_is_an_error() {
    Command::cargo_bin("rtree")
        .unwrap()
        .arg("definitely-not-a-real-rtree-path")
        .assert()
        .failure()
        .stderr(predicate::str::contains("path does not exist"));
}

#[test]
fn filters_depth_hidden_and_directories_only_work_together() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("keep")).unwrap();
    fs::create_dir(dir.path().join("keep").join("nested")).unwrap();
    fs::create_dir(dir.path().join(".secret")).unwrap();
    fs::write(dir.path().join("note.txt"), "x").unwrap();
    fs::create_dir(dir.path().join("scratch-target")).unwrap();
    Command::cargo_bin("rtree")
        .unwrap()
        .args(["-d", "-L", "1", "-I", "target"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("keep/")
                .and(predicate::str::contains("nested").not())
                .and(predicate::str::contains(".secret").not())
                .and(predicate::str::contains("note.txt").not())
                .and(predicate::str::contains("scratch-target").not()),
        );
}

#[test]
fn reverse_sort_orders_siblings_descending() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("alpha"), "x").unwrap();
    fs::write(dir.path().join("zulu"), "x").unwrap();
    Command::cargo_bin("rtree")
        .unwrap()
        .arg("--reverse")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?s)zulu.*alpha").unwrap());
}

#[test]
fn summary_reports_displayed_directory_and_file_totals() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("nested")).unwrap();
    fs::write(dir.path().join("nested").join("child.txt"), "x").unwrap();
    fs::write(dir.path().join("root.txt"), "x").unwrap();
    Command::cargo_bin("rtree")
        .unwrap()
        .arg("--summary")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("2 directories, 2 files"));
}
