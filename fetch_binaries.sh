#!/bin/sh -ex
ALPINE_VERSION=v3.8
ALPINE_MIRROR=http://dl-cdn.alpinelinux.org/alpine/
APK_TOOLS=apk-tools-static-2.10.0-r0.apk
BUSYBOX=busybox-static-1.28.4-r0.apk
ALPINE_KEYS=alpine-keys-2.1-r1.apk

ARCH=${1:-x86_64}

SHA1SUMS_x86_64="\
309a288e7730bd1e00f06fb54a8da90f35286a9e  $APK_TOOLS
09a1b792741e902a817fb9512b65692230323cc0  $BUSYBOX
4dd03fa0dfeefdd81ac13d77e0d3ed069821de33  $ALPINE_KEYS"

SHA1SUMS_armhf="\
9a19667aebd4b404c6be547c451e6c1bdd693953  $APK_TOOLS
0bc1f7bcc242ea15c4b1fcbd6faeb9b52b5046de  $BUSYBOX
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
