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

### Homebrew

```bash
brew install tonegawa07/tap/ndir
```

### Cargo

```bash
cargo install ndir
```

### Setup

Add to your `~/.zshrc`:

```zsh
eval "$(ndir --init)"
```

## Usage

| Key | Action |
|---|---|
| `↑` `↓` | Move cursor |
| `Ctrl+K` `Ctrl+J` | Move cursor (vim-style) |
| `Enter` | cd to selected directory |
| `→` | Browse into directory |
| `←` | Go back to parent |
| `Tab` | cd to current directory |
| `Esc` / `Ctrl+C` | Cancel |
| `Ctrl+H` | Toggle hidden files |
| `Ctrl+F` | Toggle file display |
| `Y` | Copy selected path to clipboard |
| Type | Fuzzy filter |
| `Backspace` | Delete filter character |

## License

MIT
