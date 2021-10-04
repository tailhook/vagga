#!/bin/sh -ex
ALPINE_VERSION=v3.14
ALPINE_MIRROR=http://dl-cdn.alpinelinux.org/alpine/
APK_TOOLS=apk-tools-static-2.12.7-r0.apk
BUSYBOX=busybox-static-1.33.1-r3.apk
ALPINE_KEYS=alpine-keys-2.3-r1.apk

ARCH=${1:-x86_64}

SHA1SUMS_x86_64="\
8799d473cab14110bc76ddcb40117a2aa1af0f52  $APK_TOOLS
a0b8d2ca9da5d9d0e89c51031ca3df9ee3da2482  $BUSYBOX
a2ed0478b872f5fdca6dc4c9f5cbcbe5624a2ef2  $ALPINE_KEYS"

SHA1SUMS_armhf="\
98174a1408561462ca6fccfc2b6870e982a6a66d  $APK_TOOLS
1e14b8eb57cca5ad96a04a5c5a1420606d9b9dd2  $BUSYBOX
1b9e1fd274ea018749d105c36e854df0ca1dc161  $ALPINE_KEYS"

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
