#!/bin/sh -ex

: ${project_root:=.}
: ${vagga_exe:=vagga}
: ${container_hash:=tmpbuildhash}
: ${container_name:=work}
: ${container_fullname:=$container_name}
: ${artifacts_dir:=$project_root/.vagga/.artifacts/$container_fullname.$container_hash}
: ${container_root:=$project_root/.vagga/.roots/$container_fullname.$container_hash}
: ${cache_dir:=$project_root/.vagga/.cache/ubuntu_debootstrap}
: ${debian_debootstrap_repo:=http://http.debian.net/debian_debootstrap}
: ${debian_debootstrap_suite:=sid}
: ${debian_debootstrap_arch:=amd64}
: ${debian_debootstrap_packages:=}

type mkdir touch
type dpkg-deb dpkg debootstrap

LD_PRELOAD=$(dirname ${vagga_exe})/inventory/libfake.so debootstrap \
    --include="${debian_debootstrap_packages}" \
    --arch "$debian_debootstrap_arch" "$debian_debootstrap_suite" "$container_root" \
    "$debian_debootstrap_repo"


