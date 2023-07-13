#!/usr/bin/env bash
# shellcheck disable=SC3037

PROGRAM=`basename $0`
script_path=$(echo $(cd $(dirname $0) && pwd))
brew_metadata_path=$(echo $(cd "${script_path}/../etc/brew" && pwd))
brew_taps_path="${brew_metadata_path}/taps.txt"
mkdir -p "$(dirname ${brew_taps_path})"
gq="$(which jq)"

fail() {
    $*
    exit 84
}
stderr() {
    1>&2 echo -e "\033[1;38;5;220m$*\033[0m"
}

show_usage() {
    why="$@"
    if [ ! -z "$why" ]; then
        stderr "\033[1;38;5;160mERROR: \033[1;38;5;184m${why}\033[0m"
    fi
    1>&2 cat <<EOF
USAGE: $PROGRAM [-fc]
    where:

    -f/--formulas prints out formulas
    -c/--casks    prints out casks"
EOF
}
fail_no_args() {
    exit 1
}
if [ -z "$1" ]; then
    fail show_usage "missing args"
fi
filter() {
    tr -d '"' | gsed 's/^null$//g'
}
string_matches_regex() {
    regex="$1"
    shift
    string="$*"
    echo "${string}" | 1>/dev/null 2>/dev/null grep "${regex}"
}

extract_and_save_metadata_from_brew() {
    kind="$1"
    if ! string_matches_regex "${kind}" "\(cask\|formula\)"; then
        fail stderr "invalid parameter ${kind}\n should be either 'cask' or 'formula'"
    fi
    plural_kind="${kind}s"
    # figlet -f small FORMULAS | gsed 's/^/  # /g'
    parent_cache_path="${brew_metadata_path}/${plural_kind}"
    mkdir -p "${parent_cache_path}"
    for keg in $(brew list "--${kind}" -rt); do
        cache_path="${parent_cache_path}/${keg}"
        mkdir -p "${cache_path}"
        keg_metadata_path="${cache_path}/metadata.json"
        if [ ! -e "${keg_metadata_path}" ]; then
            brew info --json "${keg}" > "${keg_metadata_path}"
        fi
        $gq ".[0].tap" < "${keg_metadata_path}" >> "${brew_taps_path}"
        for attribute in "full_name" "linked_keg" "homepage"; do
            $gq ".[0].${attribute}" < "${keg_metadata_path}" | filter > "${cache_path}/${attribute}"
        done
        $gq ".[0].installed[0].version" < "${keg_metadata_path}" | filter > "${cache_path}/installed.version"
        version=$(cat "${cache_path}/installed.version")
        homepage=$(cat "${cache_path}/homepage")
        name=$(cat "${cache_path}/full_name")
        echo "  depends_on ${kind}: \"${name}\" => \"${version}\"" >> "${brew_metadata_path}/${plural_kind}.txt"
    done
    sort -u "${brew_metadata_path}/${plural_kind}.txt" -o "${brew_metadata_path}/${plural_kind}.txt"
}

set -eu
if [ "$1" = "-f" ] || [ "$1" = "--formulas" ]; then
    extract_and_save_metadata_from_brew "formula"
elif [ "$1" = "-c" ] || [ "$1" = "--casks" ]; then
    extract_and_save_metadata_from_brew "cask"
else
    fail show_usage "invalid argument \033[1;38;5;154m$1"
fi

# ...TK
# formulas=0
# casks=0
# while getopts "w:,width:" opt; do
#     case $opt in
#         f|formulas)  formulas=1;;
#         c|casks)  casks=1;;
#     esac
# done
