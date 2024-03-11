# This is used for Wasabi OS
TARGET=x86_64-unknown-none
ROOT=$(shell readlink -f ../..)/generated
RUSTFLAGS=\
		  -C link-args=-e \
		  -C link-args=entry \
		  -C link-args=-z \
		  -C link-args=execstack
CARGO=RUSTFLAGS='${RUSTFLAGS}' cargo
FEATURES=--features=wasabi

.PHONY : build
build : build_saba build_httpclient

build_saba : setup
	$(CARGO) build $(FEATURES) --bin=saba --target=$(TARGET)
	$(CARGO) install $(FEATURES) --bin=saba --target=$(TARGET) --force --root $(ROOT) --path .

build_httpclient : setup
	$(CARGO) build $(FEATURES) --bin=httpclient --target=$(TARGET)
	$(CARGO) install $(FEATURES) --bin=httpclient --target=$(TARGET) --force --root $(ROOT) --path .

setup :
	rustup target add $(TARGET)
