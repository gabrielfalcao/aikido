#!/usr/bin/env bash

set -eu

execpath=$HOME/usr/libexec

for src in $(find libexec -type f); do
    name=$(basename "${src}")
    install -dSp "${src}" "${execpath}/${name}"
done

# install all conf files prefixed with `.` in the $HOME
for src in $(find etc/home -type f); do
    name=$(basename "${src}")
    install -dSp "${src}" "${HOME}/.${name}"
done

echo 'source ${HOME}/.bashrc' >> ~/.bash_profile
echo '${PATH}:${HOME}/usr/libexec' >> ~/.bashrc
