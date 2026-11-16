# rtree

`rtree` is a fast, pleasant directory-tree command written in Rust. Directories show their number of visible direct children by default, making large projects easier to scan.

```text
$ rtree examples
[2] examples/
├── [1] api/
│   └── request.json
└── README.md
```

## Install

Prerequisite: [the Rust toolchain](https://rustup.rs/).

```sh
cargo install --path .
# or, for development
cargo run -- --help
```

## Usage

```sh
rtree [OPTIONS] [PATH]
```

| Option | Description |
| --- | --- |
| `-L, --depth <N>` | Limit traversal depth; `0` prints only the root. |
| `-a, --all` | Include dotfiles and dot-directories. |
| `--no-count` | Hide directory child counts. |
| `-d, --dirs-only` | List directories only. |
| `-l, --follow-links` | Follow symbolic links while traversing. |
| `-I, --ignore <TEXT>` | Omit names containing text; repeatable. |
| `--sort <name\|modified\|none>` | Choose sibling ordering (default: `name`). |
| `-r, --reverse` | Reverse the selected ordering. |
| `--ascii` | Use portable ASCII connector characters. |
| `-s, --summary` | Print totals for displayed directories and files. |
| `-q, --quiet` | Do not print non-fatal filesystem warnings. |

Examples:

```sh
# Inspect a Rust source tree, including hidden files, two levels deep
rtree -a -L 2 src

# Fast overview where counts are not useful
rtree --no-count --sort modified .

# CI-friendly output without Unicode
rtree --ascii -d -I target -I .git

# Show totals after a filtered listing
rtree --summary --no-count src
```

## Behavior and design

- Children are sorted case-insensitively by name by default.
- Counts represent visible, filtered, printable direct children.
- Hidden entries are excluded unless `--all` is given.
- Symlinks are displayed but not followed by default, preventing accidental cycles.
- A path that cannot be read becomes a warning, so the rest of the tree remains useful.
- `--summary` totals nodes actually displayed, including the root when applicable.
- The traversal and renderer are exposed through the `rtree` library crate for embedding.

## Development

```sh
make fmt
make check
make lint
make test
make build
```

Run `make help` for the complete list. Pull requests and issue reports are welcome; see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

The repository intentionally ships with an empty `LICENSE` file until the maintainer selects one. I recommend the **MIT License** for a small, broadly reusable command-line utility: it is permissive, short, and has excellent ecosystem compatibility. Alternatively, choose Apache-2.0 if you want an explicit patent grant.
