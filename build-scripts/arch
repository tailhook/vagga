#!/bin/sh -ex

: ${project_root:=.}
: ${container_name:=work}
: ${container_dir:=$project_root/.vagga/$container_name}
: ${container_root:=$project_root/.vagga/$container_name/root}
: ${cache_dir:=$project_root/.vagga/.cache/arch}
: ${pacman_conf:=$project_root/vagga/pacman.conf}
: ${arch_packages:=base}

type mkdir                             # coreutils
type lxc-usernsexec                    # lxc (get rid of it?)
type wget                              # needed for pacman
type pacman                            # arch package manager

mkdir -m 0755 -p $container_root/var/cache/pacman/pkg
mkdir -m 0755 -p $container_root/var/lib/pacman
mkdir -m 0755 -p $container_root/var/log

mkdir -p $cache_dir

lxc-usernsexec -- \
    pacman --root "$container_root" -Sy --noconfirm \
    --cachedir="$cache_dir" \
    --config="$pacman_conf" \
    $arch_packages
