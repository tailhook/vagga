#!/bin/sh -ex
ALPINE_VERSION=v3.5
ALPINE_MIRROR=http://dl-cdn.alpinelinux.org/alpine/
APK_TOOLS=apk-tools-static-2.6.9-r0.apk
BUSYBOX=busybox-static-1.25.1-r2.apk
ALPINE_KEYS=alpine-keys-1.3-r0.apk

ARCH=${1:-x86_64}

SHA1SUMS_x86_64="\
31b29926d6be7efb389b49d9d53b557e9b25eb7c  $APK_TOOLS
c54dbe3bc0e32c056f5199909d716daab60f80ac  $BUSYBOX
f1c6e5f7209885fec5c8dd8c99446036852988a0  $ALPINE_KEYS"

SHA1SUMS_armhf="\
5d21cc3b2641bdb47231c323e9f5d736324ed425  $APK_TOOLS
d3da202e6b6bda0273fc2d31770a72c067a453aa  $BUSYBOX
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
