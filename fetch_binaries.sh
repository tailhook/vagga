#!/bin/sh -ex
mkdir alpine 2>/dev/null || true
cd alpine
rm MIRRORS.txt 2>/dev/null || true
wget http://nl.alpinelinux.org/alpine/MIRRORS.txt
mirror=$(sort --random-sort MIRRORS.txt | head -n 1)
wget -c $mirror/v3.0/main/x86_64/apk-tools-static-2.4.4-r0.apk
wget -c $mirror/v3.0/main/x86_64/busybox-static-1.22.1-r9.apk

sha1sum -c <<SHA1SUMS
ee72e8675096d820a2d12f156015e81ac078b271  apk-tools-static-2.4.4-r0.apk
dfd715ebf74132b262249a3f3efde33d071cafb6  busybox-static-1.22.1-r9.apk
SHA1SUMS
cd ..

tar -xf alpine/apk-tools-static-2.4.4-r0.apk sbin/apk.static -O > apk
tar -xf alpine/busybox-static-1.22.1-r9.apk bin/busybox.static -O > busybox

chmod +x apk busybox
