#!/bin/sh

if [ -n "$(which podman)" ]; then
    echo 'using podman' 1>&2
    echo 'podman'
elif [ -n "$(which docker)" ]; then
    echo 'using docker' 1>&2
    echo 'docker'
else
    echo 'please add podman or docker to PATH' 1>&2
    exit 1
fi

