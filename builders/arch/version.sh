#!/bin/sh -e
: "${arch_image_release:=2014.09.03}"

printenv | grep '^arch_'

if [ -n "$arch_pkgbuilds" ]; then
    for pkg in ${arch_pkgbuilds}; do
        cat $pkg/PKGBUILD
    done
fi
