build: 
  cargo build --all

build-release:
  cargo build --release --all

start-web: build-web
  npm run start --prefix ./web

build-lib:
  cargo build
build-cli:
  cargo build --package postman2openapi-cli
build-web:
  wasm-pack build --release --out-dir ./web/wasm --target bundler
  npm install --prefix ./web
  npm run build --prefix ./web
build-nodejs:
  wasm-pack build --release --out-dir ./nodejs --target nodejs

clippy:
  cargo clippy -- -D warnings

test: build test-lib test-unit test-integration test-wasm-node test-wasm-chrome test-wasm-firefox

test-lib:
  cargo test --lib
test-unit:
  cargo test --test unit_tests
test-integration:
  cargo test --test integration_tests
test-wasm-firefox:
  wasm-pack test --headless --firefox --test wasm_browser
test-wasm-chrome:
  wasm-pack test --headless --chrome --test wasm_browser
test-wasm-node:
  wasm-pack test --node --test wasm_node
