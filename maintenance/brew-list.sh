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
nstdout() {
    echo -ne "\033[1;38;5;33m$*\033[0m"
}
stdout() {
    echo -e "\033[1;38;5;33m$*\033[0m"
}

show_usage() {
    why="$@"
    if [ ! -z "$why" ]; then
        stderr "\033[1;38;5;160mERROR: \033[1;38;5;184m${why}\033[0m"
    fi
    1>&2 cat <<EOF
USAGE: $PROGRAM [-fc]
    where:

    -f/--formulae prints out formulae
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
gq() {
    2>/dev/null $gq "${*}" | filter
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
    if [ "${kind}" = "formula" ]; then
        plural_kind="formulae"
    fi

    # figlet -f small FORMULAE | gsed 's/^/  # /g'
    parent_cache_path="${brew_metadata_path}/${plural_kind}/$(hostname | cut -d. -f1 | tr '[:upper:]' '[:lower:]')"
    mkdir -p "${parent_cache_path}"
    for keg in $(brew list "--${kind}" -rt); do
        cache_path="${parent_cache_path}/${keg}"
        mkdir -p "${cache_path}"
        nstdout "processing \033[1;38;5;220m${keg}..."
        keg_metadata_v2_path="${cache_path}/metadata.v2.json"
        if [ ! -e "${keg_metadata_v2_path}" ]; then
            brew info "--${kind}" --json=v2 "${keg}" > "${keg_metadata_v2_path}"
        fi
        if [ "${kind}" = "formula" ]; then
            keg_metadata_v1_path="${cache_path}/metadata.v1.json";
            if [ ! -e "${keg_metadata_v1_path}" ]; then
                brew info "--${kind}" --json=v1 "${keg}" > "${keg_metadata_v1_path}"
            fi
        fi

        gq ".${plural_kind}[0].tap" < "${keg_metadata_v2_path}" >> "${brew_taps_path}"
        for attribute in "full_name" "full_token" "linked_keg" "homepage"; do
            gq ".${plural_kind}[0].${attribute}" < "${keg_metadata_v2_path}" > "${cache_path}/${attribute}"
        done
        gq ".${plural_kind}[0].installed[0].version,.${plural_kind}[0].version" < "${keg_metadata_v2_path}" > "${cache_path}/version.installed"
        gq ".${plural_kind}[0].version" < "${keg_metadata_v2_path}" > "${cache_path}/version.latest"
        version=$(cat "${cache_path}/version.installed")
        name=$(cat "${cache_path}/full_name")
        echo "  depends_on ${kind}: \"${name}\" => \"${version}\"" >> "${brew_metadata_path}/${plural_kind}.txt"
        stdout " \033[1;38;5;112mOK"
    done
    sort -u "${brew_metadata_path}/${plural_kind}.txt" -o "${brew_metadata_path}/${plural_kind}.txt"
    sort -u "${brew_taps_path}" -o "${brew_taps_path}"
}

set -eu
if [ "$1" = "-f" ] || [ "$1" = "--formulae" ]; then
    extract_and_save_metadata_from_brew "formula"
elif [ "$1" = "-c" ] || [ "$1" = "--casks" ]; then
    extract_and_save_metadata_from_brew "cask"
else
    fail show_usage "invalid argument \033[1;38;5;154m$1"
fi
