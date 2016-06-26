#!/bin/sh -ex
ALPINE_VERSION=v3.4
APK_TOOLS=apk-tools-static-2.6.7-r0.apk
BUSYBOX=busybox-static-1.24.2-r9.apk
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
eba31757fd5dd94f11475ab45036351ae157d260  $APK_TOOLS
63e3c39f51cf11b3acb0765609d41e932346126f  $BUSYBOX
43e2920260f598d37fe4b1157cd44f1f2581613f  $ALPINE_KEYS
SHA1SUMS
cd ..

tar -xf alpine/$APK_TOOLS sbin/apk.static -O > apk
tar -xf alpine/$BUSYBOX bin/busybox.static -O > busybox
cp alpine/$ALPINE_KEYS alpine-keys.apk

chmod +x apk busybox
