# rtree

`rtree` is a fast, pleasant directory-tree command written in Rust. Directories show their number of visible direct children by default, making large projects easier to scan. Output is colored when printed to a terminal.

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
| `-c, --no-color` | Disable colored output. |

Examples:

```sh
# Inspect a Rust source tree, including hidden files, two levels deep
rtree -a -L 2 src

# Fast overview where counts are not useful
rtree --no-count --sort modified .

# CI-friendly output without Unicode or colors
rtree --ascii --no-color -d -I target -I .git

# Show totals after a filtered listing
rtree --summary --no-count src
```

## Colors

Directories are blue, symlinks are cyan and child counts are yellow by default. Symlinks also show where they point:

```text
├── Makefile@ -> ../Makefile
```

Colors turn off automatically when output is piped or redirected, when the `NO_COLOR` environment variable is set, or with `--no-color`.

To change the colors, set `RTREE_COLORS` to a comma-separated list of `key=color` pairs:

```sh
export RTREE_COLORS="dir=magenta,link=green,count=bright black"
```

| Key | Applies to |
| --- | --- |
| `dir` | Directory names |
| `link` | Symlinks |
| `count` | The `[N]` child counts |

Accepted values are the standard terminal color names (`red`, `green`, `blue`, `magenta`, `cyan`, `white`, ...) and their `bright` variants written with a space, such as `bright blue`. Unknown keys and invalid colors are ignored.

## Behavior and design

- Children are sorted case-insensitively by name by default.
- Counts represent visible, filtered, printable direct children.
- Hidden entries are excluded unless `--all` is given.
- Symlinks are displayed with their target but not followed by default, preventing accidental cycles.
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

For manual testing there are quick targets that run against the `test_env` folder:

```sh
make demo               # tree of test_env with defaults
make demo-depth D=2     # limit depth
make demo-nocolor       # colors off
make env-links          # create sample symlinks (file, self-loop, broken)
make env-reset          # remove them again
```

Run `make help` for the complete list. Pull requests and issue reports are welcome; see [CONTRIBUTING.md](CONTRIBUTING.md).

## License
[MIT License](src/LICENSE)
