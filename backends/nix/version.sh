#!/bin/sh -ex

: ${nix_config:=default.nix}
: ${nix_attribute:=""}
: ${NIX_PATH:=${nix_path:-${CALLER_NIX_PATH}}}
: ${NIX_REMOTE:=${nix_remote:-${CALLER_NIX_REMOTE}}}

export NIX_PATH NIX_REMOTE

type nix-instantiate

nix-instantiate "${nix_config}" --attr "${nix_attribute}"
