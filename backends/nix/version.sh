#!/bin/sh -e

: ${container_name:=work}
: ${cache_dir:=.vagga/.cache}
: ${nix_config:=default.nix}
: ${nix_attribute:=""}
: ${NIX_PATH:=${nix_path:-${CALLER_NIX_PATH}}}
: ${NIX_REMOTE:=${nix_remote:-${CALLER_NIX_REMOTE}}}

export NIX_PATH NIX_REMOTE

type nix-instantiate
type test mkdir readlink

test -d $cache_dir || mkdir -p $cache_dir
# readlink is needed because if we so --add-root we get that path instead real
readlink $(
    nix-instantiate "${nix_config}" --attr "${nix_attribute}" \
    --add-root $cache_dir/$container_name.drv --indirect)
