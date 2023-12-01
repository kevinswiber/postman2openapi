prepare: test build-release build-web

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
build-web: build-js
  npm install --prefix ./web
  npm run build --prefix ./web
build-js:
  cargo build --target wasm32-unknown-unknown --release
  wasm-bindgen --out-dir ./js ./target/wasm32-unknown-unknown/release/postman2openapi.wasm
  wasm-snip --snip-rust-panicking-code -o ./js/postman2openapi_bg.wasm ./js/postman2openapi_bg.wasm
  wasm-opt -Oz -o ./js/postman2openapi_bg.wasm ./js/postman2openapi_bg.wasm
build-devcontainer-image:
  NEEDS_BUILDER=$(docker buildx ls | grep -q postman2openapi; echo $?); \
  if [[ "$NEEDS_BUILDER" = "1" ]]; then docker buildx create --name postman2openapi --bootstrap --use; \
    else docker buildx use postman2openapi; fi && \
  docker buildx build --platform linux/amd64,linux/arm64 --push -f ./.devcontainer/Dockerfile -t ghcr.io/kevinswiber/postman2openapi-devcontainer:latest .

push-devcontainer-image:
  docker push ghcr.io/kevinswiber/postman2openapi-devcontainer:latest

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
