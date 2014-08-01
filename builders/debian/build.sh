#!/bin/sh -ex

: ${vagga_exe=vagga}
: ${project_root:=.}
: ${container_hash:=tmpbuildhash}
: ${container_name:=work}
: ${container_fullname:=$container_name}
: ${artifacts_dir:=$project_root/.vagga/.artifacts/$container_fullname.$container_hash}
: ${container_root:=$project_root/.vagga/.roots/$container_fullname.$container_hash}
: ${cache_dir:=$project_root/.vagga/.cache/debian}
: ${debian_repo:=http://http.debian.net/debian/}
: ${debian_suite:=sid}
: ${debian_arch:=amd64}
: ${debian_packages:=minbase}

type mkdir touch
type dpkg-deb dpkg debootstrap

LD_PRELOAD=$(dirname $vagga_exe)/libfake.so debootstrap \
    --variant=minbase \
    --include="${debian_packages}" \
    --arch "$debian_arch" "$debian_suite" "$container_root" \
    "$debian_repo"

#mkdir -m 0755 -p $container_root/var/lib/dpkg
#mkdir -m 0755 -p $container_root/var/lib/dpkg/updates
#mkdir -m 0755 -p $container_root/var/lib/dpkg/info
#touch "$container_root/var/lib/dpkg/status"
#touch "$container_root/var/lib/dpkg/available"
#
#for i in $container_root/var/cache/apt/archives/*.deb; do
#    dpkg-deb --fsys-tarfile $i
#done | tar --ignore-zeros -xf- -C $container_root
#
#export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
#${vagga_exe} _chroot --writeable $container_root /usr/bin/fakeroot-sysv dpkg --install /var/cache/apt/archives/*.deb


