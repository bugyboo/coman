build:
	cargo build

build-release:
	cargo build --release

build-windows:
	cargo build --release --target x86_64-pc-windows-gnu

run:
	cargo run

# Maintenance commands
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Code quality commands
clippy:
	@echo "🔍 Running Clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	@echo "📝 Formatting code..."
	cargo fmt --all

fmt-check:
	@echo "📝 Checking code formatting..."
	cargo fmt --all -- --check	

test:
	./run-tests.sh