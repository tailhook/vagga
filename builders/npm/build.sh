#!/bin/sh -ex

: ${project_root:=.}
: ${container_hash:=tmpbuildhash}
: ${container_name:=work}
: ${container_fullname:=$container_name}
: ${artifacts_dir:=$project_root/.vagga/.artifacts/$container_fullname.$container_hash}
: ${container_root:=$project_root/.vagga/.roots/$container_fullname.$container_hash}
: ${cache_dir:=$project_root/.vagga/.cache/npm}
: ${npm_packages:=}
: ${npm_alpine_packages:=alpine-base nodejs git make}
: ${npm_alpine_mirror:=http://nl.alpinelinux.org/alpine/}

type mkdir
type tar

apk=$(${vagga_inventory}/fetch \
    ${npm_alpine_mirror}/v3.0/main/x86_64/apk-tools-static-2.4.4-r0.apk)

tar -xzf "${apk}" -C "${container_root}"

"${container_root}"/sbin/apk.static \
    -X "${npm_alpine_mirror}"/v3.0/main \
    -U --allow-untrusted \
    --root "${container_root}" \
    --initdb \
    add ${npm_alpine_packages}

"${vagga_exe}" _chroot --writeable \
    --environ=PATH=/bin:/usr/bin \
    --environ=HOME=/tmp \
    "$container_root" npm install --global ${npm_packages}


