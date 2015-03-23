#!/bin/sh -ex
mkdir alpine 2>/dev/null || true
cd alpine
rm MIRRORS.txt 2>/dev/null || true
wget --no-use-server-timestamp http://nl.alpinelinux.org/alpine/MIRRORS.txt -O MIRRORS.txt
mirror=$(sort --random-sort MIRRORS.txt | head -n 1)
wget --no-use-server-timestamp $mirror/v3.1/main/x86_64/apk-tools-static-2.5.0_rc1-r0.apk -O apk-tools-static-2.5.0_rc1-r0.apk
wget --no-use-server-timestamp $mirror/v3.1/main/x86_64/busybox-static-1.22.1-r15.apk -O busybox-static-1.22.1-r15.apk
wget --no-use-server-timestamp $mirror/v3.1/main/x86_64/alpine-keys-1.1-r0.apk -O alpine-keys-1.1-r0.apk

sha1sum -c <<SHA1SUMS
24900dd5945e0c3d5bc6ee8ce1b8f3d3e21430d6  apk-tools-static-2.5.0_rc1-r0.apk
7c7496702df578fcd3e2112d99ffa31e08dcfde7  busybox-static-1.22.1-r15.apk
2cbad6e762c895bfc20c3d0eb357f5d7e750da45  alpine-keys-1.1-r0.apk
SHA1SUMS
cd ..

tar -xf alpine/apk-tools-static-2.5.0_rc1-r0.apk sbin/apk.static -O > apk
tar -xf alpine/busybox-static-1.22.1-r15.apk bin/busybox.static -O > busybox
cp alpine/alpine-keys-1.1-r0.apk alpine-keys.apk

chmod +x apk busybox
