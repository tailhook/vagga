#!/bin/sh -ex

: ${container_hash:=tmpbuildhash}
: ${container_name:=work}
: ${container_fullname:=$container_name}
: ${artifacts_dir:=.vagga/.artifacts/$container_fullname.$container_hash}
: ${container_root:=.vagga/.roots/$container_fullname.$container_hash}
: ${nix_config:=default.nix}
: ${nix_attribute:=""}
: ${NIX_PATH:=${nix_path:-${CALLER_NIX_PATH}}}
: ${NIX_REMOTE:=${nix_remote:-${CALLER_NIX_REMOTE}}}

export NIX_PATH NIX_REMOTE

type mkdir mv dirname readlink            # coreutils
type nix-instantiate nix-env nix-store    # nix
type rsync                                # rsync


profile=$artifacts_dir/nix-profile
mkdir -p $artifacts_dir 2>/dev/null

test -h $profile && unlink $profile
nix-env --profile $profile --install \
    --file "${nix_config}" --attr "${nix_attribute}"
closure=$(nix-store --query --requisites $profile)

mkdir -p $container_root/nix/store
rsync --recursive --links --perms --times --hard-links \
    $closure $container_root/nix/store
rsync --recursive --links --perms --times \
    $profile/ $container_root

# few nix fixups
chmod -R u+w $container_root
mkdir -p $container_root/usr/bin
ln -sfn /bin/env $container_root/usr/bin/env
