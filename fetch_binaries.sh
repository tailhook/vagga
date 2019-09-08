#!/bin/sh -ex
ALPINE_VERSION=v3.8
ALPINE_MIRROR=http://dl-cdn.alpinelinux.org/alpine/
APK_TOOLS=apk-tools-static-2.10.1-r0.apk
BUSYBOX=busybox-static-1.28.4-r3.apk
ALPINE_KEYS=alpine-keys-2.1-r1.apk

ARCH=${1:-x86_64}

SHA1SUMS_x86_64="\
6a669b68be304249d5d3398b8db2cc5cc23674bf  $APK_TOOLS
9538e3a393d1e39e55037ab10187c48f43ec971f  $BUSYBOX
4dd03fa0dfeefdd81ac13d77e0d3ed069821de33  $ALPINE_KEYS"

SHA1SUMS_armhf="\
b30f18f5743d13f1845577e98e629a93c49212a4  $APK_TOOLS
63093a34579a68b95d13a3f44149babb2b0f07bc  $BUSYBOX
a10caf9d8162d5ca16fc77729cfebf9c79d8c87b  $ALPINE_KEYS"

FETCH_DIR="alpine/"$ARCH
mkdir -p "$FETCH_DIR" 2>/dev/null || true
cd "$FETCH_DIR"

for pkg in $APK_TOOLS $BUSYBOX $ALPINE_KEYS; do
    wget --no-use-server-timestamp ${ALPINE_MIRROR}${ALPINE_VERSION}/main/$ARCH/$pkg -O $pkg
done

sha1sum $APK_TOOLS
sha1sum $BUSYBOX
sha1sum $ALPINE_KEYS
SUMS="SHA1SUMS_$ARCH"
eval "SUMS=\$$SUMS"
echo "$SUMS" | sha1sum -c -

cd ../..

tar -xOf "$FETCH_DIR/$APK_TOOLS" sbin/apk.static > apk
tar -xOf "$FETCH_DIR/$BUSYBOX" bin/busybox.static > busybox
cp "$FETCH_DIR/$ALPINE_KEYS" alpine-keys.apk

chmod +x apk busybox
