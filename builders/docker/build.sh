#!/bin/sh -xe
: ${project_root:=.}
: ${vagga_inventory:=/usr/lib/vagga/inventory}
: ${container_hash:=tmpbuildhash}
: ${container_name:=work}
: ${container_fullname:=$container_name}
: ${artifacts_dir:=$project_root/.vagga/.artifacts/$container_fullname.$container_hash}
: ${container_root:=$project_root/.vagga/.roots/$container_fullname.$container_hash}
: ${cache_dir:=$project_root/.vagga/.cache/from_image}
: ${docker_image:=ubuntu}

type curl
type mkdir

if [ "${docker_image#*/}" = "$docker_image" ]; then
    repo="library/$docker_image";
else
    repo="$docker_image"
fi
if [ "${repo%:*}" = "$repo" ]; then
    tag=latest
else
    repo="${repo%:*}"
    tag="${docker_image#*:}"
fi

mkdir $artifacts_dir
curl --header "X-Docker-Token: true" --output $artifacts_dir/tags.json \
    --dump-header $artifacts_dir/tags_header.txt --insecure \
    https://index.docker.io/v1/repositories/$repo/tags
