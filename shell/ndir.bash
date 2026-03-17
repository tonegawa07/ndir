function ndir() {
    local tmpfile=$(mktemp /tmp/ndir.XXXXXX)
    command ndir "$@" > "$tmpfile"
    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        local result=$(<"$tmpfile")
        if [[ -n "$result" && -d "$result" ]]; then
            builtin cd -- "$result"
        fi
    fi
    rm -f "$tmpfile"
}

# Ctrl+N keybinding
bind -x '"\C-n": ndir .'
