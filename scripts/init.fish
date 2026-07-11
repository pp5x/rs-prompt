# rs-prompt fish integration.
#
# This file is also embedded by `rs-prompt init fish`.

function __rs_prompt_detect_vcs_dir
    set -l dir "$argv[1]"
    if test -d "$dir/.jj"; or test -L "$dir/.jj"
        return 0
    end
    if test -d "$dir/.git"; or test -L "$dir/.git"; or test -f "$dir/.git"
        return 0
    end
    if test -d "$dir/.repo"; or test -L "$dir/.repo"
        return 0
    end
    return 1
end

function __rs_prompt_outermost_vcs_root
    set -l dir "$PWD"

    while test "$dir" != "/" -a -n "$dir"
        if __rs_prompt_detect_vcs_dir "$dir"
            printf '%s\n' "$dir"
            return
        end

        set dir (string replace -r '/[^/]*$' '' -- "$dir")
        if test -z "$dir"
            set dir /
        end
    end
end

function cd
    if test (count $argv) -ne 0
        builtin cd $argv
        return
    end

    set -l root (__rs_prompt_outermost_vcs_root)
    if test -n "$root"
        builtin cd "$root"
    else
        builtin cd
    end
end

function __rs_prompt_render_prompt
    __RS_PROMPT_BIN__ prompt --shell=fish --prompt-escape=fish --status="$argv[1]"
end

set -gx VIRTUAL_ENV_DISABLE_PROMPT 1

function fish_prompt
    set -l rs_prompt_status $status
    __rs_prompt_render_prompt "$rs_prompt_status"
end
