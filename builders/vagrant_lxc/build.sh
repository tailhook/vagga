#!/bin/sh -ex

: ${project_root:=.}
: ${vagga_inventory:=/usr/lib/vagga/inventory}
: ${container_hash:=tmpbuildhash}
: ${container_name:=work}
: ${container_fullname:=$container_name}
: ${artifacts_dir:=$project_root/.vagga/.artifacts/$container_fullname.$container_hash}
: ${container_root:=$project_root/.vagga/.roots/$container_fullname.$container_hash}
: ${cache_dir:=$project_root/.vagga/.cache/vagrant_lxc}
: ${vagrant_lxc_name:=fgrehm/trusty64-lxc}
: ${vagrant_lxc_url:=https://vagrantcloud.com/${vagrant_lxc_name}/version/1/provider/lxc.box}

type basename
type bsdtar

rmdir $container_root
mkdir -p $artifacts_dir

path=$($vagga_inventory/fetch "$vagrant_lxc_url")
tar -xf $path -C "$artifacts_dir/"

tar -xf $artifacts_dir/rootfs.tar.* \
    --no-same-owner --exclude=rootfs/dev/* -C $artifacts_dir
mv $artifacts_dir/rootfs/ $container_root
