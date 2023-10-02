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

fmt-check:
  cargo fmt --check --all
clippy:
  cargo clippy -- -D warnings

test: build fmt-check clippy test-lib test-integration test-wasm-node test-wasm-chrome test-wasm-firefox

test-lib:
  cargo test --lib
test-integration:
  cargo test --test integration_tests
test-wasm-firefox:
  (which geckodriver && wasm-pack test --headless --firefox --test wasm_browser) || echo "Install geckodriver to run Firefox tests."
test-wasm-chrome:
  (which chromedriver && wasm-pack test --headless --chrome --test wasm_browser) || echo "Install chromedriver to run Chrome tests."
test-wasm-node:
  wasm-pack test --node --test wasm_node
