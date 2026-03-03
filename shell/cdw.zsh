# cdw: inline directory navigation
# Source this file in your .zshrc

# cdw function: run binary, capture result from tmpfile, then cd
function cdw() {
    local tmpfile=$(mktemp /tmp/cdw.XXXXXX)
    "${CDW_BIN:-cdw}" "$@" > "$tmpfile"
    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        local result=$(<"$tmpfile")
        if [[ -n "$result" && -d "$result" ]]; then
            builtin cd -- "$result"
        fi
    fi
    rm -f "$tmpfile"
}

# Short alias
alias n='cdw'

# Ctrl+N keybinding
function _cdw_widget() {
    cdw .
    zle reset-prompt
}
zle -N _cdw_widget
bindkey '^N' _cdw_widget
