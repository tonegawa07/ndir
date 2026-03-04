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

```bash
cargo install ndir
```

Add to your `~/.zshrc`:

```zsh
eval "$(ndir --init)"
```

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
