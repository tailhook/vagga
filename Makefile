all: vagga

vagga: quire argparse src/*.rs
	rustc src/mod.rs -L rust-quire -L rust-argparse -o $@

quire:
	make -C rust-quire quire-lib

argparse:
	make -C rust-argparse argparse-lib

.PHONE: all quire argparse
