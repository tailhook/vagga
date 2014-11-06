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

vagga: rust-argparse/libargparse.rlib rust-quire/libquire.rlib libconfig.rlib
vagga: src/launcher/*.rs
	$(RUSTC) src/launcher/main.rs -g -o $@ \
		-L rust-quire -L rust-argparse -L .


.PHONY: all
