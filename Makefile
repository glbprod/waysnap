# Makefile for waysnap
#
# Targets:
#   make build          — debug build (fast)
#   make release        — optimized native build
#   make cross          — cross-compile for aarch64 (Raspberry Pi 64-bit)
#   make install        — install binary to ~/.local/bin
#   make uninstall      — remove binary from ~/.local/bin
#   make test           — run unit tests
#   make run-install    — cargo run -- install (patch rc.xml + reload labwc)
#   make show-config    — print the XML snippet without modifying anything

BINARY      := waysnap
CARGO       := cargo
INSTALL_DIR := $(HOME)/.local/bin
CROSS_TARGET := aarch64-unknown-linux-gnu

.PHONY: build release cross install uninstall test run-install show-config clean

## Default target
build:
	$(CARGO) build

## Optimized native build
release:
	$(CARGO) build --release

## Cross-compile for aarch64 (Raspberry Pi 64-bit Raspbian)
## Requires: rustup target add aarch64-unknown-linux-gnu
##           apt install gcc-aarch64-linux-gnu (or equivalent cross toolchain)
cross:
	$(CARGO) build --release --target $(CROSS_TARGET)

## Install native release binary to ~/.local/bin
install: release
	@mkdir -p $(INSTALL_DIR)
	cp target/release/$(BINARY) $(INSTALL_DIR)/$(BINARY)
	@echo "Installed to $(INSTALL_DIR)/$(BINARY)"

## Install cross-compiled aarch64 binary to ~/.local/bin
install-cross: cross
	@mkdir -p $(INSTALL_DIR)
	cp target/$(CROSS_TARGET)/release/$(BINARY) $(INSTALL_DIR)/$(BINARY)
	@echo "Installed aarch64 binary to $(INSTALL_DIR)/$(BINARY)"

## Remove installed binary
uninstall:
	rm -f $(INSTALL_DIR)/$(BINARY)
	@echo "Removed $(INSTALL_DIR)/$(BINARY)"

## Run unit tests
test:
	$(CARGO) test

## Patch ~/.config/labwc/rc.xml and reload labwc (requires LABWC_PID to be set)
run-install:
	$(CARGO) run -- install

## Print the XML snippet without modifying any file
show-config:
	$(CARGO) run -- show-config

## Remove build artifacts
clean:
	$(CARGO) clean
