# ndir: inline directory navigation
# Source this file in your .zshrc

function ndir() {
    local tmpfile=$(mktemp /tmp/ndir.XXXXXX)
    "${NDIR_BIN:-ndir}" "$@" > "$tmpfile"
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
function _ndir_widget() {
    ndir .
    zle reset-prompt
}
zle -N _ndir_widget
bindkey '^N' _ndir_widget
