#!/bin/sh -ex
mkdir alpine 2>/dev/null || true
cd alpine
rm MIRRORS.txt 2>/dev/null || true
wget http://nl.alpinelinux.org/alpine/MIRRORS.txt
mirror=$(sort --random-sort MIRRORS.txt | head -n 1)
wget -c $mirror/v3.1/main/x86_64/apk-tools-static-2.5.0_rc1-r0.apk
wget -c $mirror/v3.1/main/x86_64/busybox-static-1.22.1-r14.apk

sha1sum -c <<SHA1SUMS
24900dd5945e0c3d5bc6ee8ce1b8f3d3e21430d6  apk-tools-static-2.5.0_rc1-r0.apk
744354c9edd8fd855b8c40724da9922a6f434fd1  busybox-static-1.22.1-r14.apk
SHA1SUMS
cd ..

tar -xf alpine/apk-tools-static-2.5.0_rc1-r0.apk sbin/apk.static -O > apk
tar -xf alpine/busybox-static-1.22.1-r14.apk bin/busybox.static -O > busybox

chmod +x apk busybox
