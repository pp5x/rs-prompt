#!/bin/sh

set -eu

binary=${1:-target/release/rs-prompt}
iterations=${RS_PROMPT_BENCH_ITERATIONS:-2000}
host=${RS_PROMPT_BENCH_HOST:-build42.lab}
user=${RS_PROMPT_BENCH_USER:-alice}

if [ ! -x "$binary" ]; then
    echo "benchmark binary is not executable: $binary" >&2
    exit 1
fi

run_case() {
    name=$1
    cwd=$2
    home=$3
    escape=$4

    printf '%s (%s iterations)\n' "$name" "$iterations"
    /usr/bin/time -p sh -c '
        i=0
        while [ "$i" -lt "$1" ]; do
            HOME="$3" "$5" prompt \
                --cwd="$2" \
                --host="$6" \
                --user="$7" \
                --shell=zsh \
                --prompt-escape="$4" >/dev/null
            i=$((i + 1))
        done
    ' sh "$iterations" "$cwd" "$home" "$escape" "$binary" "$host" "$user"
}

run_case "root" / /home/example none
run_case "shallow non-VCS" /tmp /home/example none
run_case "repository" "$(pwd)/src" /Users/pp5x none
run_case "repository with zsh escaping" "$(pwd)/src" /Users/pp5x zsh

printf '%s\n' "init script emission ($iterations iterations)"
/usr/bin/time -p sh -c '
    i=0
    while [ "$i" -lt "$1" ]; do
        "$2" init zsh >/dev/null
        i=$((i + 1))
    done
' sh "$iterations" "$binary"
