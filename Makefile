RUSTC ?= rustc
CC ?= gcc
AR ?= ar


export PREFIX ?= /usr
export DESTDIR ?=
export VAGGA_VERSION = $(shell git describe)
PACKAGE_FILES = vagga apk busybox alpine-keys.apk install.sh
COMPLETION_FILES = completions/bash-completion.sh completions/zsh-completion.sh


all: downloads vagga

with-docker: downloads
	cargo build --no-default-features --features docker_runner

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
	&& mkdir -p $$tmpdir/vagga/completions \
	&& cp $(PACKAGE_FILES) $$tmpdir/vagga \
	&& cp $(COMPLETION_FILES) $$tmpdir/vagga/completions \
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
