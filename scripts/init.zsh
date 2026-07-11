# rs-prompt zsh integration.
#
# This file is also embedded by `rs-prompt init zsh`.

prompt_rs_prompt_precmd() {
    RS_PROMPT_STATUS=$?
}

__rs_prompt_detect_vcs_dir() {
    local dir="$1"
    [[ -d "$dir/.jj" || -L "$dir/.jj" ]] && return 0
    [[ -d "$dir/.git" || -L "$dir/.git" || -f "$dir/.git" ]] && return 0
    [[ -d "$dir/.repo" || -L "$dir/.repo" ]] && return 0
    return 1
}

__rs_prompt_outermost_vcs_root() {
    local dir="$PWD"
    local root=""

    while [[ "$dir" != "/" && -n "$dir" ]]; do
        if __rs_prompt_detect_vcs_dir "$dir"; then
            root="$dir"
        fi
        dir="${dir%/*}"
    done

    [[ -n "$root" ]] && print -r -- "$root"
}

cd() {
    if (( $# != 0 )); then
        builtin cd "$@"
        return
    fi

    local root="$(__rs_prompt_outermost_vcs_root)"
    if [[ -n "$root" ]]; then
        builtin cd "$root"
    else
        builtin cd
    fi
}

autoload -Uz add-zsh-hook
add-zsh-hook precmd prompt_rs_prompt_precmd

__rs_prompt_render_prompt() {
    __RS_PROMPT_BIN__ prompt --shell=zsh --prompt-escape=zsh --status="${RS_PROMPT_STATUS:-0}"
}

VIRTUAL_ENV_DISABLE_PROMPT=1
setopt promptsubst

PROMPT='$(__rs_prompt_render_prompt)'
