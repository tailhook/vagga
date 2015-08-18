RUSTC ?= rustc
CC ?= gcc
AR ?= ar
CARGO_FLAGS ?=


export PREFIX ?= /usr
export DESTDIR ?=


all: downloads vagga

vagga:
	cargo build --target=x86_64-unknown-linux-musl $(CARGO_FLAGS)
	cp --remove-destination target/x86_64-unknown-linux-musl/debug/vagga .

vagga_test: tests/*.rs tests/*/*.rs
	$(RUSTC) tests/lib.rs -g --test -o $@ -L . -L rust-quire/target/release

downloads: apk busybox alpine-keys.apk

alpine/MIRRORS.txt apk busybox alpine-keys: ./fetch_binaries.sh
	./fetch_binaries.sh

test: all vagga_test
	./vagga_test

install:
	./install.sh

tarball:
	[ -d dist ] || mkdir dist
	tarname=vagga-$$(git describe | cut -c2-).tar.xz \
	&& tmpdir=$$(mktemp -d) \
	&& mkdir -p $$tmpdir/vagga \
	&& cp vagga apk busybox alpine-keys.apk install.sh $$tmpdir/vagga \
	&& tar -cJf dist/$$tarname -C $$tmpdir/vagga \
		vagga apk busybox alpine-keys.apk install.sh \
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


.PHONY: all downloads test vagga install
