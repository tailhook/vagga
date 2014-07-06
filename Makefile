all: vagga

vagga: rust-quire src/*.rs
	rustc src/mod.rs -L rust-quire -L rust-quire/rust-argparse -o $@

rust-quire:
	make -C rust-quire

.PHONE: all rust-quire
