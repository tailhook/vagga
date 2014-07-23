RUSTC ?= rustc
CC ?= gcc

all: vagga

vagga: quire argparse src/*.rs libcontainer.a
	$(RUSTC) src/mod.rs -L rust-quire -L rust-argparse -g -o $@

libcontainer.a: container.c
	$(CC) -c $< -o $@

quire:
	make -C rust-quire quire-lib

argparse:
	make -C rust-argparse argparse-lib

.PHONE: all quire argparse
