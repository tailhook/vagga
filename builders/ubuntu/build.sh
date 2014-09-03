#!/bin/sh -ex

: ${project_root:=.}
: ${vagga_inventory:=/usr/lib/vagga/inventory}
: ${vagga_exe:=vagga}
: ${container_hash:=tmpbuildhash}
: ${container_name:=work}
: ${container_fullname:=$container_name}
: ${artifacts_dir:=$project_root/.vagga/.artifacts/$container_fullname.$container_hash}
: ${container_root:=$project_root/.vagga/.roots/$container_fullname.$container_hash}
: ${cache_dir:=$project_root/.vagga/.cache/ubuntu}
: ${ubuntu_kind:=core}
: ${ubuntu_release:=trusty}
: ${ubuntu_arch:=amd64}
: ${ubuntu_initial_image:=http://cdimage.ubuntu.com/ubuntu-${ubuntu_kind}/${ubuntu_release}/daily/current/${ubuntu_release}-${ubuntu_kind}-${ubuntu_arch}.tar.gz}
: ${ubuntu_PPAs:=}
: ${ubuntu_repos:=}
: ${ubuntu_initial_packages:=}
: ${ubuntu_additional_repos:=}
: ${ubuntu_additional_keys:=}
: ${ubuntu_packages:=}

type tar sed cut sha1sum awk

mkdir -p $container_root
mkdir -p $artifacts_dir
mkdir -p $cache_dir

mkdir -p $cache_dir/apt-cache
mkdir -p $container_root/var/cache/apt

path=$($vagga_inventory/fetch $ubuntu_initial_image)
chroot="${vagga_exe} _chroot \
        --writeable --inventory \
        --volume "$cache_dir/apt-cache:/var/cache/apt:rw" \
        --environ PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
        --environ DEBIAN_FRONTEND=noninteractive \
        --environ LANG=${CALLER_LANG} \
        --environ TERM=${CALLER_TERM}"
if test "$(awk '{total += $3} END {print total}' /proc/self/uid_map)" -lt 2; then
    echo "Warning you have no mapped uids. You should probably add some" >&2
    echo "But we will try to fix it using libfake" >&2
    tar_flags="--no-same-owner"
    chroot="$chroot --environ=LD_PRELOAD=/tmp/inventory/libfake.so"
fi
chroot="$chroot $container_root"

tar -xf $path $tar_flags --exclude 'dev/*' -C $container_root

# prevent init scripts from running
echo $'#!/bin/sh\nexit 101' > "$container_root/usr/sbin/policy-rc.d"
chmod +x "$container_root/usr/sbin/policy-rc.d"
echo 'force-unsafe-io' > "$container_root/etc/dpkg/dpkg.cfg.d/02apt-speedup"

if test -n "$ubuntu_PPAs" || test -n "$ubuntu_additional_repos"; then
    ubuntu_initial_packages="${ubuntu_initial_packages} software-properties-common"
fi

if test -n "$ubuntu_additional_repos"; then
    for repo in ${ubuntu_additional_repos}; do
        if test "${repo#https:}" != "${repo}"; then
            ubuntu_initial_packages="${ubuntu_initial_packages} apt-transport-https"
        fi
        repo="$(echo $repo | sed 's/|/ /g')"
        id="$(echo $repo | sha1sum | cut -c1-8)"
        echo "deb $repo" > "$container_root/etc/apt/sources.list.d/${id}.list"
    done
fi

if test -n "${ubuntu_initial_packages}"; then
    $chroot apt-get -y install ${ubuntu_initial_packages}
fi

if test -n "$ubuntu_repos"; then
    for repo in "${ubuntu_repos}"; do
        sed -i '/'$repo'/{s/^# //;}' "${container_root}/etc/apt/sources.list"
    done
fi

if test -n "$ubuntu_PPAs"; then
    for ppa in ${ubuntu_PPAs}; do
        $chroot apt-add-repository -y ppa:$ppa
    done
fi

if test -n "$ubuntu_additional_keys"; then
    $chroot apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 \
        --recv-keys ${ubuntu_additional_keys}
fi

if test -n "$ubuntu_packages"; then
    $chroot apt-get update
    $chroot apt-get -y install ${ubuntu_packages}
fi

