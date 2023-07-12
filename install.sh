#!/usr/bin/env bash

set -eu

execpath=$HOME/usr/libexec

for src in $(find libexec -type f); do
    name=$(basename "${src}")
    echo install "${src}" "${execpath}/${name}"
done
