#!/bin/sh -ex

: ${project_root:=.}
: ${container_hash:=tmpbuildhash}
: ${container_name:=work}
: ${container_fullname:=$container_name}
: ${artifacts_dir:=$project_root/.vagga/.artifacts/$container_fullname.$container_hash}
: ${container_root:=$project_root/.vagga/.roots/$container_fullname.$container_hash}
: ${cache_dir:=$project_root/.vagga/.cache/arch}
: ${arch_pacman_conf:=$(dirname $0)/pacman.conf}
: ${arch_packages:=base}

type mkdir                             # coreutils
type wget                              # needed for pacman
type pacman                            # arch package manager

mkdir -m 0755 -p $container_root/var/cache/pacman/pkg
mkdir -m 0755 -p $container_root/var/lib/pacman
mkdir -m 0755 -p $container_root/var/log

mkdir -p $cache_dir

pacman --root "$container_root" -Sy --noconfirm \
    --cachedir="$cache_dir" \
    --config="$arch_pacman_conf" \
    $arch_packages
