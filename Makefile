# This is used for Wasabi OS
TARGET=x86_64-unknown-none
ROOT=$(shell readlink -f ../../generated)
RUSTFLAGS=\
		  -C link-args=-e \
		  -C link-args=entry \
		  -C link-args=-z \
		  -C link-args=execstack
CARGO=RUSTFLAGS='${RUSTFLAGS}' cargo

.PHONY : build
build :
	rustup target add $(TARGET)
	$(CARGO) build --features=wasabi --bin=toybr --target=$(TARGET)
	$(CARGO) install --force --root $(ROOT) --path .
