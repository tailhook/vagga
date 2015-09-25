RUSTC ?= rustc
CC ?= gcc
AR ?= ar
CARGO_FLAGS ?=


export PREFIX ?= /usr
export DESTDIR ?=


all: downloads vagga

release: downloads vagga-release

vagga:
	cargo build --target=x86_64-unknown-linux-musl
	cp --remove-destination target/x86_64-unknown-linux-musl/debug/vagga .

vagga-release:
	cargo build --target=x86_64-unknown-linux-musl --release
	cp --remove-destination target/x86_64-unknown-linux-musl/release/vagga .

downloads: apk busybox alpine-keys.apk

alpine/MIRRORS.txt apk busybox alpine-keys: ./fetch_binaries.sh
	./fetch_binaries.sh

install:
	./install.sh

tarball:
	[ -d dist ] || mkdir dist
	tarname=vagga-$$(git describe | cut -c2-).tar.xz \
	&& tmpdir=$$(mktemp -d) \
	&& mkdir -p $$tmpdir/vagga \
	&& cp vagga apk busybox alpine-keys.apk install.sh $$tmpdir/vagga \
	&& tar -cJf dist/$$tarname -C $$tmpdir vagga \
	&& rm -rf $$tmpdir \
	echo Done tarball $$tarname


ubuntu-package:
	checkinstall \
		--default \
		--maintainer=paul@colomiets.name \
		--pkglicense=MIT \
		--pkgname=vagga \
		--pkgver="$$(git describe | cut -c2-)" \
		--pakdir="dist" \
		--requires="uidmap" \
		--backup=no \
		--nodoc \
		$(CHECKINSTALL_FLAGS) \
		./install.sh

.PHONY: all downloads vagga install release vagga-release
