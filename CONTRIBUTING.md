# Contributing to rtree

Thanks for considering a contribution. Small, focused changes with tests are easiest to review.

## Local setup

Install the stable Rust toolchain through [rustup](https://rustup.rs/), fork the repository, then run:

```sh
git clone <your-fork-url>
cd rtree
make check
make test
```

## Development workflow

1. Create a branch with a descriptive name.
2. Keep the public CLI behavior documented in `README.md`.
3. Add or update unit tests for traversal/rendering behavior and integration tests for CLI behavior.
4. Run `make fmt lint test` before opening a pull request.
5. Describe user-visible behavior and any platform differences in the pull request.

## Design expectations

- Preserve deterministic output unless a user explicitly requests unsorted output.
- Treat inaccessible descendant paths as recoverable warnings where possible.
- Avoid following symlinks by default.
- Keep dependencies small and cross-platform.
- Prefer the library API for logic; keep `main.rs` limited to command-line wiring.

## Reporting bugs

Please include your platform, `rtree --version`, the command run, expected versus actual output, and a small reproducible directory layout. Do not include private filesystem paths or sensitive filenames.

## Code of conduct

Be respectful, constructive, and welcoming. Harassment and personal attacks are not acceptable.
