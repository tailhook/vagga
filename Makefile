RUSTC ?= rustc
CC ?= gcc
AR ?= ar


PREFIX ?= /usr
DESTDIR ?=


all: vagga_launcher vagga_wrapper vagga_test

rust-argparse/libargparse.rlib:
	make -C rust-argparse libargparse.rlib

rust-quire/libquire.rlib:
	make -C rust-quire libquire.rlib

libconfig.rlib: src/config/*.rs
	$(RUSTC) src/config/lib.rs -g -o $@ \
		-L rust-quire -L rust-argparse

container.o: container.c
	$(CC) -c $< -o $@ -fPIC -D_GNU_SOURCE -std=c99

libcontainer.a: container.o
	$(AR) rcs $@ $^

libcontainer.rlib: src/container/*.rs libcontainer.a
	$(RUSTC) src/container/lib.rs -g -o $@ -L . -L rust-quire

vagga_launcher: rust-argparse/libargparse.rlib rust-quire/libquire.rlib
vagga_launcher: libconfig.rlib libcontainer.rlib
vagga_launcher: src/launcher/*.rs
	$(RUSTC) src/launcher/main.rs -g -o $@ \
		-L rust-quire -L rust-argparse -L .

vagga_wrapper: rust-argparse/libargparse.rlib rust-quire/libquire.rlib
vagga_wrapper: libconfig.rlib libcontainer.rlib
vagga_wrapper: src/wrapper/*.rs
	$(RUSTC) src/wrapper/main.rs -g -o $@ \
		-L rust-quire -L rust-argparse -L .

vagga_test: tests/*.rs tests/*/*.rs
	$(RUSTC) tests/lib.rs -g --test -o $@ -L . -L rust-quire


.PHONY: all
