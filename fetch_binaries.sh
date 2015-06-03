#!/bin/sh -ex
ALPINE_VERSION=v3.2
APK_TOOLS=apk-tools-static-2.6.1-r0.apk
BUSYBOX=busybox-static-1.23.2-r0.apk
ALPINE_KEYS=alpine-keys-1.1-r0.apk


mkdir alpine 2>/dev/null || true
cd alpine
rm MIRRORS.txt 2>/dev/null || true
wget --no-use-server-timestamp http://nl.alpinelinux.org/alpine/MIRRORS.txt -O MIRRORS.txt
mirror=$(sort --random-sort MIRRORS.txt | head -n 1)
wget --no-use-server-timestamp $mirror/$ALPINE_VERSION/main/x86_64/$APK_TOOLS -O $APK_TOOLS
wget --no-use-server-timestamp $mirror/$ALPINE_VERSION/main/x86_64/$BUSYBOX -O $BUSYBOX
wget --no-use-server-timestamp $mirror/$ALPINE_VERSION/main/x86_64/$ALPINE_KEYS -O $ALPINE_KEYS

sha1sum -c <<SHA1SUMS
d9ad6ad8ede8ff554b49a084516990b75df4563b  $APK_TOOLS
ff581245a7291bb13a642a4a90704829bf6cbbbb  $BUSYBOX
036260ac2053a02048a57703c62c0730e63b0d79  $ALPINE_KEYS
SHA1SUMS
cd ..

tar -xf alpine/$APK_TOOLS sbin/apk.static -O > apk
tar -xf alpine/$BUSYBOX bin/busybox.static -O > busybox
cp alpine/$ALPINE_KEYS alpine-keys.apk

chmod +x apk busybox
