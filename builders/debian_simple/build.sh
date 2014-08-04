#!/bin/sh -ex

: ${project_root:=.}
: ${container_hash:=tmpbuildhash}
: ${container_name:=work}
: ${container_fullname:=$container_name}
: ${artifacts_dir:=$project_root/.vagga/.artifacts/$container_fullname.$container_hash}
: ${container_root:=$project_root/.vagga/.roots/$container_fullname.$container_hash}
: ${cache_dir:=$project_root/.vagga/.cache/debian_simple}
: ${debian_simple_repo:=http://http.debian.net/debian/}
: ${debian_simple_suite:=sid}
: ${debian_simple_arch:=amd64}
: ${debian_simple_packages:=minbase}

type mkdir touch
type dpkg-deb dpkg debootstrap

debootstrap \
    --download-only \
    --include="${debian_simple_packages}" \
    --arch "$debian_simple_arch" "$debian_simple_suite" "$container_root" \
    "$debian_simple_repo"

mkdir -m 0755 -p $container_root/var/lib/dpkg
mkdir -m 0755 -p $container_root/var/lib/dpkg/updates
mkdir -m 0755 -p $container_root/var/lib/dpkg/info
touch "$container_root/var/lib/dpkg/status"
touch "$container_root/var/lib/dpkg/available"

for i in $container_root/var/cache/apt/archives/*.deb; do
    dpkg-deb --fsys-tarfile $i;
done  |  tar --ignore-zeros -xf- -C $container_root


