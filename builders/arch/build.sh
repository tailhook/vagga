#!/bin/sh -ex

project_root="${project_root:-.}"
container_hash="${container_hash:-tmpbuildhash}"
container_name="${container_name:-work}"
container_fullname="${container_fullname:-$container_name}"
artifacts_dir="${artifacts_dir:-$project_root/.vagga/.artifacts/$container_fullname.$container_hash}"
container_root="${container_root:-$project_root/.vagga/.roots/$container_fullname.$container_hash}"
cache_dir="${cache_dir:-$project_root/.vagga/.cache/arch}"
arch_mirror="${arch_mirror:-http://mirror.de.leaseweb.net/archlinux/}"
arch_arch="${arch_arch:-x86_64}"
arch_image_release="${arch_image_release:-2014.09.03}"
arch_initial_image="${arch_mirror}/iso/${arch_image_release}/archlinux-bootstrap-${arch_image_release}-${arch_arch}.tar.gz"
arch_packages="${arch_packages:-base}"
arch_pkgbuilds="${arch_pkgbuilds:-}"
arch_build_dependencies="${arch_build_dependencies:-base-devel}"
arch_build_nocheck="${arch_build_nocheck:-}"
arch_additional_repos="${arch_additional_repos:-}"

type grep tar sed cut sha1sum awk

mkdir -p $container_root
mkdir -p $artifacts_dir
mkdir -p $cache_dir

mkdir -p $cache_dir/src
mkdir -p $artifacts_dir/builddir
mkdir -p $cache_dir/pacman-cache
mkdir -p $container_root/var/cache/pacman

path=$($vagga_inventory/fetch $arch_initial_image)
chroot="${vagga_exe} _chroot \
        --writeable --inventory \
        --volume "$cache_dir/pacman-cache:/var/cache/pacman:rw" \
        --environ PATH=/bin \
        --environ SRCDEST=/work/.vagga/.cache/arch/src \
        --environ LANG=${CALLER_LANG} \
        --environ TERM=${CALLER_TERM}"

if test "$(awk '{total += $3} END {print total}' /proc/self/uid_map)" -lt 2; then
    echo "Warning you have no mapped uids. You should probably add some" >&2
    echo "But we will try to fix it using libfake" >&2
    tar_flags="--no-same-owner"
    chroot="$chroot --environ=LD_PRELOAD=/tmp/inventory/libfake.so"
fi
chroot="$chroot $container_root"

tar -xf $path --strip-components=1 $tar_flags \
    --exclude 'dev/*' -C $container_root

sed -i '\|'"$arch_mirror"'|{s/^#//;}' "$container_root/etc/pacman.d/mirrorlist"
sed -i '/^SigLevel/{s/.*/SigLevel = Never/;}' "$container_root/etc/pacman.conf"

$chroot pacman-key --init
$chroot pacman -Sy --noconfirm archlinux-keyring

sed -i '/^SigLevel/{s/.*/SigLevel = Required DatabaseOptional/;}' \
    "$container_root/etc/pacman.conf"

for repo in ${arch_additional_repos}; do
    name="$(echo $repo | cut -d'|' -f1)"
    url="$(echo $repo | cut -d'|' -f2)"
    cat <<END >> "$container_root/etc/pacman.conf"
[$name]
SigLevel = Never
Server = $url
END
done

$chroot pacman -Syu --noconfirm
$chroot pacman -S --noconfirm ${arch_packages}

if [ -n "$arch_pkgbuilds" ]; then
    $chroot pacman -S --asdeps --noconfirm ${arch_build_dependencies}
    for pkg in ${arch_pkgbuilds}; do
        cp -r "$pkg" "$artifacts_dir/builddir/"
        (cd "$artifacts_dir/builddir/$(basename "$pkg")" \
            && $chroot makepkg ${arch_build_nocheck:+--nocheck} \
                               --noconfirm --asroot --install)
    done
    $chroot pacman -D --asexplicit --noconfirm ${arch_packages}
    to_delete=$($chroot pacman -Qq --deps --unrequired)
    test -n "$to_delete" \
        && $chroot pacman -R --unneeded --recursive --noconfirm $to_delete
fi
