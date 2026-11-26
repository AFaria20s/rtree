use crate::{Node, NodeKind, Tree, TreeOptions};
use colored::{ColoredString, Colorize};
use std::io::{self, Write};

pub fn write_tree<W: Write>(writer: &mut W, tree: &Tree, options: &TreeOptions) -> io::Result<()> {
    writeln!(writer, "{}", label(&tree.root, options))?;
    write_children(writer, &tree.root, "", options)?;
    if options.show_summary {
        let summary = summary(&tree.root);
        writeln!(
            writer,
            "\n{} {}, {} {}",
            summary.directories,
            pluralize(summary.directories, "directory", "directories"),
            summary.files,
            pluralize(summary.files, "file", "files"),
        )?;
    }
    if !options.quiet {
        for warning in &tree.warnings {
            writeln!(writer, "warning: {warning}")?;
        }
    }
    Ok(())
}

#[derive(Default)]
struct Summary {
    directories: usize,
    files: usize,
}

fn summary(node: &Node) -> Summary {
    let mut result = Summary::default();
    match node.kind {
        NodeKind::Directory => result.directories += 1,
        NodeKind::File => result.files += 1,
        NodeKind::Symlink | NodeKind::Other => {}
    }
    for child in &node.children {
        let child_summary = summary(child);
        result.directories += child_summary.directories;
        result.files += child_summary.files;
    }
    result
}

fn pluralize<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

fn write_children<W: Write>(
    writer: &mut W,
    node: &Node,
    prefix: &str,
    options: &TreeOptions,
) -> io::Result<()> {
    let (tee, elbow, pipe, space) = if options.ascii {
        ("|- ", "`- ", "|  ", "   ")
    } else {
        ("├── ", "└── ", "│   ", "    ")
    };
    for (index, child) in node.children.iter().enumerate() {
        let last = index + 1 == node.children.len();
        writeln!(
            writer,
            "{prefix}{}{}",
            if last { elbow } else { tee },
            label(child, options)
        )?;
        let next_prefix = format!("{prefix}{}", if last { space } else { pipe });
        write_children(writer, child, &next_prefix, options)?;
    }
    Ok(())
}
fn paint(text: &str, options: &TreeOptions, style: impl FnOnce(&str) -> ColoredString) -> String {
    if options.no_color {
        text.to_string()
    } else {
        style(text).to_string()
    }
}

fn label(node: &Node, options: &TreeOptions) -> String {
    if node.is_symlink {
        let name = paint(&format!("{}@", node.name), options, |s| {
            s.color(options.theme.symlink)
        });
        return match &node.link_target {
            Some(target) => format!("{name} -> {}", target.display()),
            None => name,
        };
    }

    match node.kind {
        NodeKind::Directory => {
            let name = paint(&format!("{}/", node.name), options, |s| {
                s.color(options.theme.directory).bold()
            });
            if options.show_counts {
                let count = paint(&format!("[{}]", node.child_count), options, |s| {
                    s.color(options.theme.count).bold()
                });
                format!("{count} {name}")
            } else {
                name
            }
        }
        _ => node.name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeKind, Tree};
    use std::path::PathBuf;
    fn node(name: &str, kind: NodeKind, children: Vec<Node>) -> Node {
        Node {
            name: name.into(),
            path: PathBuf::from(name),
            kind,
            child_count: children.len(),
            children,
            is_symlink: false,
            link_target: None,
            modified: None,
        }
    }
    #[test]
    fn followed_symlink_keeps_marker_and_target() {
        let mut link = node(
            "dirlink",
            NodeKind::Directory,
            vec![node("x", NodeKind::File, vec![])],
        );
        link.is_symlink = true;
        link.link_target = Some(PathBuf::from("real"));

        let tree = Tree {
            root: node("root", NodeKind::Directory, vec![link]),
            warnings: vec![],
        };
        let mut output = Vec::new();
        tree.write_to(
            &mut output,
            &TreeOptions {
                no_color: true,
                show_counts: false,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "root/\n└── dirlink@ -> real\n    └── x\n"
        );
    }
    #[test]
    fn renders_counts_and_connectors() {
        let tree = Tree {
            root: node(
                "root",
                NodeKind::Directory,
                vec![
                    node("a", NodeKind::File, vec![]),
                    node(
                        "dir",
                        NodeKind::Directory,
                        vec![node("b", NodeKind::File, vec![])],
                    ),
                ],
            ),
            warnings: vec![],
        };
        let mut output = Vec::new();
        tree.write_to(
            &mut output,
            &TreeOptions {
                no_color: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "[2] root/\n├── a\n└── [1] dir/\n    └── b\n"
        );
    }
    #[test]
    fn ascii_mode_and_no_counts_render() {
        let tree = Tree {
            root: node(
                "root",
                NodeKind::Directory,
                vec![node("child", NodeKind::Directory, vec![])],
            ),
            warnings: vec![],
        };
        let mut output = Vec::new();
        tree.write_to(
            &mut output,
            &TreeOptions {
                ascii: true,
                show_counts: false,
                no_color: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "root/\n`- child/\n");
    }

    #[test]
    fn summary_counts_displayed_directories_and_files() {
        let tree = Tree {
            root: node(
                "root",
                NodeKind::Directory,
                vec![
                    node("file", NodeKind::File, vec![]),
                    node(
                        "nested",
                        NodeKind::Directory,
                        vec![node("other", NodeKind::File, vec![])],
                    ),
                ],
            ),
            warnings: vec![],
        };
        let mut output = Vec::new();
        tree.write_to(
            &mut output,
            &TreeOptions {
                show_summary: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            String::from_utf8(output)
                .unwrap()
                .ends_with("\n2 directories, 2 files\n")
        );
    }
}
