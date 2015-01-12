RUSTC ?= rustc
CC ?= gcc
AR ?= ar


PREFIX ?= /usr
DESTDIR ?=


all: vagga_launcher vagga_wrapper vagga_version vagga_build vagga_setup_netns
all: vagga_network


rust-argparse/libargparse.rlib:
	make -C rust-argparse libargparse.rlib

rust-quire/libquire.rlib:
	make -C rust-quire libquire.rlib

libconfig.rlib: src/config/*.rs rust-quire/libquire.rlib
	$(RUSTC) src/config/lib.rs -g -o $@ \
		-L rust-quire -L rust-argparse

container.o: container.c
	$(CC) -c $< -o $@ -fPIC -D_GNU_SOURCE -std=c99

libcontainer.a: container.o
	$(AR) rcs $@ $^

libcontainer.rlib: src/container/*.rs libcontainer.a libconfig.rlib
	$(RUSTC) src/container/lib.rs -g -o $@ -L . -L rust-quire

rust_compile_static = \
	$(RUSTC) -o $(1).o --emit obj $(2); \
	rlibs=$$($(RUSTC) -Z print-link-args $(2) \
		| tr -s " '" '\n' | grep rlib) \
	&& gcc -static -static-libgcc $(1).o $$rlibs -o $(1) \
		-lpthread -lm -ldl -lrt -lutil

vagga_launcher: rust-argparse/libargparse.rlib rust-quire/libquire.rlib
vagga_launcher: libconfig.rlib libcontainer.rlib
vagga_launcher: src/launcher/*.rs
	$(call rust_compile_static,$@,src/launcher/main.rs -g \
		-L rust-quire -L rust-argparse -L .)

vagga_wrapper: rust-argparse/libargparse.rlib rust-quire/libquire.rlib
vagga_wrapper: libconfig.rlib libcontainer.rlib
vagga_wrapper: src/wrapper/*.rs
	$(call rust_compile_static,$@,src/wrapper/main.rs -g \
		-L rust-quire -L rust-argparse -L .)

vagga_version: rust-argparse/libargparse.rlib rust-quire/libquire.rlib
vagga_version: libconfig.rlib libcontainer.rlib
vagga_version: src/version/*.rs
	$(call rust_compile_static,$@,src/version/main.rs -g \
		-L rust-quire -L rust-argparse -L .)

vagga_build: rust-argparse/libargparse.rlib rust-quire/libquire.rlib
vagga_build: libconfig.rlib libcontainer.rlib
vagga_build: src/builder/*.rs src/builder/commands/*.rs
	$(call rust_compile_static,$@,src/builder/main.rs -g \
		-L rust-quire -L rust-argparse -L .)

vagga_setup_netns: rust-argparse/libargparse.rlib
vagga_setup_netns: src/setup_netns/*.rs
	$(call rust_compile_static,$@,src/setup_netns/main.rs -g -L rust-argparse)

vagga_network: rust-argparse/libargparse.rlib rust-quire/libquire.rlib
vagga_network: libconfig.rlib libcontainer.rlib
vagga_network: src/network/*.rs
	$(call rust_compile_static,$@,src/network/main.rs -g \
		-L rust-argparse -L rust-quire -L .)

vagga_test: tests/*.rs tests/*/*.rs
	$(RUSTC) tests/lib.rs -g --test -o $@ -L . -L rust-quire

test: all vagga_test
	./vagga_test


.PHONY: all
