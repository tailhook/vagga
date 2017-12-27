#!/bin/sh -ex
ALPINE_VERSION=v3.5
ALPINE_MIRROR=http://dl-cdn.alpinelinux.org/alpine/
APK_TOOLS=apk-tools-static-2.6.9-r0.apk
BUSYBOX=busybox-static-1.25.1-r1.apk
ALPINE_KEYS=alpine-keys-1.3-r0.apk


mkdir alpine 2>/dev/null || true
cd alpine

for pkg in $APK_TOOLS $BUSYBOX $ALPINE_KEYS; do
    wget --no-use-server-timestamp ${ALPINE_MIRROR}${ALPINE_VERSION}/main/x86_64/$pkg -O $pkg
done

sha1sum $APK_TOOLS
sha1sum $BUSYBOX
sha1sum $ALPINE_KEYS
sha1sum -c - <<SHA1SUMS
31b29926d6be7efb389b49d9d53b557e9b25eb7c  $APK_TOOLS
8b5639a22af1e656f03931ab38ef51285b3b9dd2  $BUSYBOX
f1c6e5f7209885fec5c8dd8c99446036852988a0  $ALPINE_KEYS
SHA1SUMS
cd ..

tar -xOf alpine/$APK_TOOLS sbin/apk.static > apk
tar -xOf alpine/$BUSYBOX bin/busybox.static > busybox
cp alpine/$ALPINE_KEYS alpine-keys.apk

chmod +x apk busybox
