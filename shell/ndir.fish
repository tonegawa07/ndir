function ndir
    set -l tmpfile (mktemp /tmp/ndir.XXXXXX)
    command ndir $argv > $tmpfile
    set -l exit_code $status
    if test $exit_code -eq 0
        set -l result (cat $tmpfile)
        if test -n "$result" -a -d "$result"
            builtin cd -- $result
        end
    end
    rm -f $tmpfile
end

# Ctrl+N keybinding
bind \cn 'ndir .; commandline -f repaint'
