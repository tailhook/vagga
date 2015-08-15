RUSTC ?= rustc
CC ?= gcc
AR ?= ar


PREFIX ?= /usr
DESTDIR ?=


all: downloads vagga

vagga:
	cargo build --target=x86_64-unknown-linux-musl
	cp --remove-destination target/x86_64-unknown-linux-musl/debug/vagga .

vagga_test: tests/*.rs tests/*/*.rs
	$(RUSTC) tests/lib.rs -g --test -o $@ -L . -L rust-quire/target/release

downloads: apk busybox alpine-keys.apk

alpine/MIRRORS.txt apk busybox alpine-keys: ./fetch_binaries.sh
	./fetch_binaries.sh

test: all vagga_test
	./vagga_test

install:
	install -d $(DESTDIR)$(PREFIX)/bin
	install -d $(DESTDIR)$(PREFIX)/lib/vagga
	install -m 755 vagga $(DESTDIR)$(PREFIX)/lib/vagga/vagga
	install -m 755 apk $(DESTDIR)$(PREFIX)/lib/vagga/apk
	install -m 755 busybox $(DESTDIR)$(PREFIX)/lib/vagga/busybox
	install -m 755 alpine-keys.apk $(DESTDIR)$(PREFIX)/lib/vagga/alpine-keys.apk
	ln -s ../lib/vagga/vagga $(DESTDIR)$(PREFIX)/bin/vagga


.PHONY: all downloads test vagga install
