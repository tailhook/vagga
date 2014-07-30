RUSTC ?= rustc
CC ?= gcc
ARGPARSELIB = rust-argparse/$(shell rustc --crate-file-name rust-argparse/argparse/mod.rs)
QUIRELIB = rust-quire/$(shell rustc --crate-file-name rust-quire/quire/mod.rs)

all: vagga

vagga: $(ARGPARSELIB) $(QUIRELIB) src/*.rs src/*/*.rs libcontainer.a
	$(RUSTC) src/mod.rs -L rust-quire -L rust-argparse -g -o $@

libcontainer.a: container.c
	$(CC) -c $< -o $@

$(QUIRELIB):
	make -C rust-quire quire-lib

$(ARGPARSELIB):
	make -C rust-argparse argparse-lib

.PHONE: all quire argparse
