#!/bin/sh -ex
ALPINE_VERSION=v3.5
APK_TOOLS=apk-tools-static-2.6.8-r2.apk
BUSYBOX=busybox-static-1.25.1-r0.apk
ALPINE_KEYS=alpine-keys-1.3-r0.apk


mkdir alpine 2>/dev/null || true
cd alpine
rm MIRRORS.txt 2>/dev/null || true
wget --no-use-server-timestamp http://dl-cdn.alpinelinux.org/alpine/MIRRORS.txt -O MIRRORS.txt

# OS X doesn't have --random-sort
mirror=$(head -n 1 MIRRORS.txt)
wget --no-use-server-timestamp ${mirror}$ALPINE_VERSION/main/x86_64/$APK_TOOLS -O $APK_TOOLS
wget --no-use-server-timestamp ${mirror}$ALPINE_VERSION/main/x86_64/$BUSYBOX -O $BUSYBOX
wget --no-use-server-timestamp ${mirror}$ALPINE_VERSION/main/x86_64/$ALPINE_KEYS -O $ALPINE_KEYS

sha1sum -c - <<SHA1SUMS
4f863f28867fc7100e422f39bd918f6b120c5fc5  $APK_TOOLS
b609218d7b0a1c9ec2e457c7665db8b703c4ef10  $BUSYBOX
f1c6e5f7209885fec5c8dd8c99446036852988a0  $ALPINE_KEYS
SHA1SUMS
cd ..

tar -xOf alpine/$APK_TOOLS sbin/apk.static > apk
tar -xOf alpine/$BUSYBOX bin/busybox.static > busybox
cp alpine/$ALPINE_KEYS alpine-keys.apk

chmod +x apk busybox
