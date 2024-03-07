.PHONY: all clean

TARGETS := x86_64-pc-windows-gnu x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

all: $(TARGETS)

$(TARGETS):
	cargo build --release --target $@

build:
	cargo build --release

windows-x86_64:
	cargo build --release --target x86_64-pc-windows-gnu

linux-x86_64:
	cargo build --release --target x86_64-unknown-linux-gnu

linux-aarch64:
	cargo build --release --target aarch64-unknown-linux-gnu

darwin-x86_64:
	cargo build --release --target x86_64-apple-darwin

darwin-aarch64:
	cargo build --release --target aarch64-apple-darwin

clean:
	cargo clean
	rm -rf target/
