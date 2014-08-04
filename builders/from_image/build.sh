#!/bin/sh -ex

: ${project_root:=.}
: ${container_hash:=tmpbuildhash}
: ${container_name:=work}
: ${container_fullname:=$container_name}
: ${artifacts_dir:=$project_root/.vagga/.artifacts/$container_fullname.$container_hash}
: ${container_root:=$project_root/.vagga/.roots/$container_fullname.$container_hash}
: ${cache_dir:=$project_root/.vagga/.cache/from_image}
: ${from_image_url:=http://cdimage.ubuntu.com/ubuntu-core/trusty/daily/current/trusty-core-amd64.tar.gz}

type basename
type wget
type tar

mkdir -p $container_root
mkdir -p $artifacts_dir

filename="$artifacts_dir/$(basename $from_image_url)"
wget $from_image_url -O $filename

tar -xf $filename --exclude 'dev/*' -C $container_root
