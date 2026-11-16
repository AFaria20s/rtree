use crate::{Node, NodeKind, Tree, TreeOptions};
use std::io::{self, Write};

pub fn write_tree<W: Write>(writer: &mut W, tree: &Tree, options: &TreeOptions) -> io::Result<()> {
    writeln!(writer, "{}", label(&tree.root, options))?;
    write_children(writer, &tree.root, "", options)?;
    if !options.quiet {
        for warning in &tree.warnings {
            writeln!(writer, "warning: {warning}")?;
        }
    }
    Ok(())
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
fn label(node: &Node, options: &TreeOptions) -> String {
    match node.kind {
        NodeKind::Directory => {
            if options.show_counts {
                format!("[{}] {}/", node.child_count, node.name)
            } else {
                format!("{}/", node.name)
            }
        }
        NodeKind::Symlink => format!("{}@", node.name),
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
            modified: None,
        }
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
        tree.write_to(&mut output, &TreeOptions::default()).unwrap();
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
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "root/\n`- child/\n");
    }
}
