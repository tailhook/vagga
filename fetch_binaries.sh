#!/bin/sh -ex
ALPINE_VERSION=v3.3
APK_TOOLS=apk-tools-static-2.6.5-r1.apk
BUSYBOX=busybox-static-1.24.1-r7.apk
ALPINE_KEYS=alpine-keys-1.1-r0.apk


mkdir alpine 2>/dev/null || true
cd alpine
rm MIRRORS.txt 2>/dev/null || true
wget --no-use-server-timestamp http://nl.alpinelinux.org/alpine/MIRRORS.txt -O MIRRORS.txt

# Temporarily remove non-working mirror
sed -i /lax-noc.com/D MIRRORS.txt

mirror=$(sort --random-sort MIRRORS.txt | head -n 1)
wget --no-use-server-timestamp ${mirror}$ALPINE_VERSION/main/x86_64/$APK_TOOLS -O $APK_TOOLS
wget --no-use-server-timestamp ${mirror}$ALPINE_VERSION/main/x86_64/$BUSYBOX -O $BUSYBOX
wget --no-use-server-timestamp ${mirror}$ALPINE_VERSION/main/x86_64/$ALPINE_KEYS -O $ALPINE_KEYS

sha1sum -c <<SHA1SUMS
5ad5503eb198fcb8620a31488166bb3e263a1dbe  $APK_TOOLS
daf93dba541e8b720bbda1495078b18a13c9ec91  $BUSYBOX
77c9ddbe28843434e2d7dd8b393f5041efd82090  $ALPINE_KEYS
SHA1SUMS
cd ..

tar -xf alpine/$APK_TOOLS sbin/apk.static -O > apk
tar -xf alpine/$BUSYBOX bin/busybox.static -O > busybox
cp alpine/$ALPINE_KEYS alpine-keys.apk

chmod +x apk busybox
