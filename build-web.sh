wasm-pack build --release --out-dir ./web/wasm --target bundler
cd web
npm install
npm run build
