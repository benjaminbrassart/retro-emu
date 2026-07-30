RUSTDIR := /tmp/rust
export RUSTUP_HOME := ${RUSTDIR}/.rustup
export CARGO_HOME := ${RUSTDIR}/.cargo
export PATH := ${CARGO_HOME}/bin:${PATH}

FLAGS := --release --target-dir /tmp/.target

release:
	@if [ ! -d ${RUSTDIR} ]; then \
		unset RUSTUP_HOME; \
		unset CARGO_HOME; \
	fi; \
	cargo run ${FLAGS}

dev:
	@if [ ! -d ${RUSTDIR} ]; then \
		unset RUSTUP_HOME; \
		unset CARGO_HOME; \
	fi; \
	cargo run

rust-install:
	@RUSTUP_HOME=${RUSTUP_HOME} \
	CARGO_HOME=${CARGO_HOME} \
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
	sh -s -- -y --profile minimal

rust-clean:
	@rustup self uninstall
	@rm -rf ${RUSTDIR}

.PHONY: release dev rust-install rust-clean
