#!/bin/sh -e
: ${project_root}
: ${docker_image:=}
: ${docker_dockerfile:=}

echo ${docker_image}
if [ -n "${docker_dockerfile}" ]; then
    # Contents of the dockerfile matters not the name
    cat ${docker_dockerfile}
fi
