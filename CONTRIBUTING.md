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

1. Update `CHANGELOG.md`: move items from `[Unreleased]` to `[x.y.z] - YYYY-MM-DD`
2. Update `version` in `Cargo.toml`
3. Commit: `git commit -m "Release vx.y.z"`
4. Tag and push: `git tag vx.y.z && git push && git push --tags`
5. CI will verify version consistency and publish to crates.io

## Bug Reports

Please file a [bug report](https://github.com/tonegawa07/ndir/issues/new?template=bug_report.md) with reproduction steps.

## Feature Requests

Please file a [feature request](https://github.com/tonegawa07/ndir/issues/new?template=feature_request.md) describing your use case.
