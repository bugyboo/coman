build:
	cargo build

build-release:
	cargo build --release

build-windows:
	cargo build --release --target x86_64-pc-windows-gnu

run:
	cargo run

test:
	./run-tests.sh