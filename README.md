# ndir

Navigate directories inline in your shell. Browse and cd without leaving your prompt.

Unlike full-screen file managers (yazi, nnn, ranger), ndir renders **inline** — right below your prompt.

## Demo

```
~/projects $ ndir

  ~/projects
 > frontend/
   backend/
   scripts/
   docs/
  enter:cd  tab:here  arrows:navigate  esc:quit
```

## Install

### Build from source

Requires Rust toolchain.

```bash
git clone https://github.com/tonegawa07/ndir.git
cd ndir
cargo build --release
```

Binary will be at `target/release/ndir`.

### Shell setup (zsh)

Add to your `~/.zshrc`:

```zsh
export NDIR_BIN="/path/to/ndir"
source /path/to/ndir/shell/ndir.zsh
```

This gives you:
- `ndir` command to launch the navigator
- `Ctrl+N` keybinding

## Usage

| Key | Action |
|---|---|
| `↑` `↓` | Move cursor |
| `Enter` | cd to selected directory |
| `→` | Browse into directory |
| `←` | Go back to parent |
| `Tab` | cd to current directory |
| `Esc` | Cancel |
| `Ctrl+H` | Toggle hidden files |
| Type | Fuzzy filter |
| `Backspace` | Delete filter character |

## License

MIT
