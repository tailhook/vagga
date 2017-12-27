#!/bin/sh -ex
ALPINE_VERSION=v3.5
ALPINE_MIRROR=http://dl-cdn.alpinelinux.org/alpine/
APK_TOOLS=apk-tools-static-2.6.9-r0.apk
BUSYBOX=busybox-static-1.25.1-r1.apk
ALPINE_KEYS=alpine-keys-1.3-r0.apk

ARCH=${1:-x86_64}

SHA1SUMS_x86_64="\
31b29926d6be7efb389b49d9d53b557e9b25eb7c  $APK_TOOLS
8b5639a22af1e656f03931ab38ef51285b3b9dd2  $BUSYBOX
f1c6e5f7209885fec5c8dd8c99446036852988a0  $ALPINE_KEYS"

SHA1SUMS_armhf="\
5d21cc3b2641bdb47231c323e9f5d736324ed425  $APK_TOOLS
63201684a82918bf3d92c122c0679681ec880bf6  $BUSYBOX
1f3acc333767f75529a66353052cec3813c127c5  $ALPINE_KEYS"

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
