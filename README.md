# cdw

Inline directory navigator for your shell. Browse and cd into directories without leaving your prompt.

Unlike full-screen file managers (yazi, nnn, ranger), cdw renders **inline** — right below your prompt.

## Demo

```
~/projects $ n

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
git clone https://github.com/tonegawa07/cdw.git
cd cdw
cargo build --release
```

Binary will be at `target/release/cdw`.

### Shell setup (zsh)

Add to your `~/.zshrc`:

```zsh
export CDW_BIN="/path/to/cdw"
source /path/to/cdw/shell/cdw.zsh
```

This gives you:
- `n` command to launch the navigator
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
