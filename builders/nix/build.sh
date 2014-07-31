#!/bin/sh -ex

: ${container_hash:=tmpbuildhash}
: ${container_name:=work}
: ${container_fullname:=$container_name}
: ${artifacts_dir:=$project_root/.vagga/.artifacts/$container_fullname.$container_hash}
: ${container_root:=$project_root/.vagga/.roots/$container_fullname.$container_hash}
: ${cache_dir:=$project_root/.vagga/.cache/nix}
: ${nix_config:=default.nix}
: ${nix_attribute:=""}
: ${NIX_PATH:=${nix_path:-${CALLER_NIX_PATH}}}
: ${NIX_REMOTE:=${nix_remote:-${CALLER_NIX_REMOTE}}}

export NIX_PATH NIX_REMOTE

type mkdir mv dirname readlink            # coreutils
type nix-instantiate nix-env nix-store    # nix
type rsync                                # rsync

test -d $cache_dir || mkdir -p $cache_dir

root=$(nix-build "${nix_config}" --attr "${nix_attribute}" \
    --out-link $cache_dir/$container_name/output \
    --drv-link $cache_dir/$container_name/derivation)
closure=$(nix-store --query --requisites $root)

mkdir -p $container_root/nix/store
rsync --recursive --links --perms --times --hard-links \
    $closure $container_root/nix/store
rsync --recursive --links --perms --times \
    $root/ $container_root

# few nix fixups
chmod -R u+w $container_root
mkdir -p $container_root/usr/bin
ln -sfn /bin/env $container_root/usr/bin/env
