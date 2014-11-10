RUSTC ?= rustc
CC ?= gcc
AR ?= ar


PREFIX ?= /usr
DESTDIR ?=


all: vagga

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
	$(RUSTC) src/container/lib.rs -g -o $@ -L .

vagga: rust-argparse/libargparse.rlib rust-quire/libquire.rlib
vagga: libconfig.rlib libcontainer.rlib
vagga: src/launcher/*.rs
	$(RUSTC) src/launcher/main.rs -g -o $@ \
		-L rust-quire -L rust-argparse -L .


.PHONY: all
