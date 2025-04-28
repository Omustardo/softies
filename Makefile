# Build targets
.PHONY: build run web clean web-prod

# Development build (fast)
build: build-native build-wasm-dev

build-native:
	cargo build

build-wasm-dev:
	wasm-pack build --target web --dev

# Production build (optimized)
build-wasm-prod:
	wasm-pack build --target web --release

run: build-native
	cargo run

# Kill any existing server on port 4000
kill-server:
	@-pkill -f "basic-http-server" || true
	@sleep 1

web: build-wasm-dev kill-server
	basic-http-server & sleep 2 && google-chrome http://localhost:4000

web-prod: build-wasm-prod kill-server
	basic-http-server & sleep 2 && google-chrome http://localhost:4000

clean:
	cargo clean
	rm -rf pkg/ 