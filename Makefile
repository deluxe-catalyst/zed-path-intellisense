.PHONY: all build-lsp build-extension clean

all: build-lsp build-extension

build-lsp:
	cd zed-path-intellisense-lsp && cargo build --release

build-extension:
	cd zed-path-intellisense && cargo build --target wasm32-wasip2 --release

clean:
	cd zed-path-intellisense-lsp && cargo clean
	cd zed-path-intellisense && cargo clean
