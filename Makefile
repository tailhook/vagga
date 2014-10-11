RUSTC ?= rustc
CC ?= gcc

PREFIX ?= /usr
DESTDIR ?=
VAGGA_PATH_DEFAULT ?= $(PREFIX)/lib/vagga
NIX_PROFILES_SUPPORT ?= yes
export VAGGA_PATH_DEFAULT

ARGPARSELIB = rust-argparse/$(shell rustc --crate-file-name rust-argparse/argparse/mod.rs)
QUIRELIB = rust-quire/$(shell rustc --crate-file-name rust-quire/src/lib.rs)

all: quire argparse vagga libfake

vagga: $(ARGPARSELIB) $(QUIRELIB) src/*.rs src/*/*.rs libcontainer.a
	$(RUSTC) src/mod.rs -g -o $@ \
		-L rust-quire -L rust-argparse \
		$(if $(NIX_PROFILES_SUPPORT),--cfg nix_profiles,)

libcontainer.a: container.c
	$(CC) -c $< -o $@

libfake: inventory/libfake.so

inventory/libfake.so: fake.c
	$(CC) -fPIC -shared -ldl $< -o $@

quire:
	make -C rust-quire quire-lib

argparse:
	make -C rust-argparse argparse-lib

install:
	install -d $(DESTDIR)$(PREFIX)/bin
	install -d $(DESTDIR)$(PREFIX)/lib/vagga
	install -m 755 vagga $(DESTDIR)$(PREFIX)/bin/vagga

	cp -r builders inventory $(DESTDIR)$(PREFIX)/lib/vagga/


.PHONY: all quire argparse libfake
