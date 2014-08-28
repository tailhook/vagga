#!/bin/sh -ex

: ${project_root:=.}
: ${vagga_inventory:=/usr/lib/vagga/inventory}
: ${container_hash:=tmpbuildhash}
: ${container_name:=work}
: ${container_fullname:=$container_name}
: ${artifacts_dir:=$project_root/.vagga/.artifacts/$container_fullname.$container_hash}
: ${container_root:=$project_root/.vagga/.roots/$container_fullname.$container_hash}
: ${cache_dir:=$project_root/.vagga/.cache/from_image}
: ${from_image_url:=http://cdimage.ubuntu.com/ubuntu-core/trusty/daily/current/trusty-core-amd64.tar.gz}

type tar

mkdir -p $container_root
mkdir -p $artifacts_dir

path=$($vagga_inventory/fetch $from_image_url)

tar -xf $path --no-same-owner --exclude 'dev/*' -C $container_root
