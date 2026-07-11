# rs-prompt bash integration.
#
# This file is also embedded by `rs-prompt init bash`.

__rs_prompt_previous_prompt_command="${PROMPT_COMMAND-}"

__rs_prompt_restore_status() {
    return "$1"
}

__rs_prompt_prompt_command() {
    local status=$?

    if [[ -n "$__rs_prompt_previous_prompt_command" ]]; then
        __rs_prompt_restore_status "$status"
        eval "$__rs_prompt_previous_prompt_command"
    fi

    RS_PROMPT_STATUS=$status
    PS1="$(__rs_prompt_render_prompt)"
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

    while [[ "$dir" != "/" && -n "$dir" ]]; do
        if __rs_prompt_detect_vcs_dir "$dir"; then
            printf '%s\n' "$dir"
            return
        fi
        dir="${dir%/*}"
    done
}

cd() {
    if (( $# != 0 )); then
        builtin cd "$@"
        return
    fi

    local root
    root="$(__rs_prompt_outermost_vcs_root)"
    if [[ -n "$root" ]]; then
        builtin cd "$root"
    else
        builtin cd
    fi
}

__rs_prompt_render_prompt() {
    __RS_PROMPT_BIN__ prompt --shell=bash --prompt-escape=bash --status="${RS_PROMPT_STATUS:-0}"
}

VIRTUAL_ENV_DISABLE_PROMPT=1
PROMPT_COMMAND=__rs_prompt_prompt_command
