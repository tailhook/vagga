#!/bin/sh -ex
ALPINE_VERSION=v3.6
APK_TOOLS=apk-tools-static-2.7.2-r0.apk
BUSYBOX=busybox-static-1.26.2-r5.apk
ALPINE_KEYS=alpine-keys-2.1-r1.apk


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
1d6caa8b3de821cdd1b52d9aca3b5de1b1f7693a  $APK_TOOLS
c6a1adc786ded36e659fa2c67c120ab482411872  $BUSYBOX
ec493688096c83625da9f7d81eed3d71d8102ba8  $ALPINE_KEYS
SHA1SUMS
cd ..

tar -xOf alpine/$APK_TOOLS sbin/apk.static > apk
tar -xOf alpine/$BUSYBOX bin/busybox.static > busybox
cp alpine/$ALPINE_KEYS alpine-keys.apk

chmod +x apk busybox
