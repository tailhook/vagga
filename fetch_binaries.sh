#!/bin/sh -ex
ALPINE_VERSION=v3.15
ALPINE_MIRROR=http://dl-cdn.alpinelinux.org/alpine/
APK_TOOLS=apk-tools-static-2.12.7-r3.apk
BUSYBOX=busybox-static-1.34.1-r5.apk
ALPINE_KEYS=alpine-keys-2.4-r1.apk

ARCH=${1:-x86_64}

SHA1SUMS_x86_64="\
2fa49548020eb850e0a15df03471a07eba55fbc8  $APK_TOOLS
15e73d6e7ae87f71b86bbf4185da0108375de29c  $BUSYBOX
7dba809ae84d5832473f9cbf3bc6522d341299ca  $ALPINE_KEYS"

SHA1SUMS_armhf="\
49fd9c34731593f5753fbc100dbb344e3f22cf47  $APK_TOOLS
9c5aa6fcfa691d720cf19a31fc855e6e0dab1689  $BUSYBOX
1c45ddb6ae0a0aee7793505cce4fcee0d82c7ac1  $ALPINE_KEYS"

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
