# Contributing

Contributions to ndir are welcome!

## Development Setup

```bash
git clone https://github.com/tonegawa07/ndir.git
cd ndir
cargo build
```

## How to Contribute

1. Open an issue to discuss what you'd like to do
2. Fork the repo and create a branch (`git checkout -b feature/my-feature`)
3. Commit your changes
4. Push and open a Pull Request

## Code Quality

Please run the following before submitting a PR:

```bash
cargo test
cargo clippy
cargo fmt --check
```

## Manual Testing

Since ndir is a TUI app, automated testing of the UI is limited. Please verify the following manually before submitting a PR:

1. `↑` `↓` to move cursor (also `Ctrl+K` / `Ctrl+J`)
2. `→` to enter a directory, `←` to go back to parent
3. Type characters to fuzzy filter, `Backspace` to delete
4. `Ctrl+H` to toggle hidden files
5. `Ctrl+F` to toggle file display
6. `Y` to copy the selected path to clipboard
7. `Enter` to cd into selected directory
8. `Tab` to cd into current directory
9. `Esc` to cancel

```bash
cargo run
```

## Releasing (for maintainers)

### Version numbering

This project follows [Semantic Versioning](https://semver.org/) (`MAJOR.MINOR.PATCH`):

- **PATCH** (e.g. 0.4.3 → 0.4.4): Bug fixes, docs, internal improvements
- **MINOR** (e.g. 0.4.4 → 0.5.0): New features (backwards compatible)
- **MAJOR** (e.g. 0.x → 1.0.0): Breaking changes or stable release declaration

### Release steps

```bash
# 1. Update CHANGELOG.md: move [Unreleased] items to [x.y.z] - YYYY-MM-DD
# 2. Update version in Cargo.toml
# 3. Update Cargo.lock
cargo check
# 4. Commit
git add CHANGELOG.md Cargo.toml Cargo.lock
git commit -m "Release vx.y.z"
# 5. Tag and push
git tag vx.y.z
git push && git push --tags
```

CI will automatically:
- Verify that the tag, `Cargo.toml`, and `CHANGELOG.md` versions match
- Publish to crates.io
- Build binaries for macOS (x86_64, aarch64) and Linux (x86_64)
- Create a GitHub Release with the binaries
- Update the Homebrew formula with new checksums

## Bug Reports

Please file a [bug report](https://github.com/tonegawa07/ndir/issues/new?template=bug_report.md) with reproduction steps.

## Feature Requests

Please file a [feature request](https://github.com/tonegawa07/ndir/issues/new?template=feature_request.md) describing your use case.
