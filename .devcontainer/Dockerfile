FROM mcr.microsoft.com/devcontainers/rust:latest

RUN apt-get update
RUN apt-get install -y ca-certificates curl gnupg
RUN mkdir -p /etc/apt/keyrings
RUN curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | sudo gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg

RUN echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_20.x nodistro main" | sudo tee /etc/apt/sources.list.d/nodesource.list
RUN apt-get update && apt-get install nodejs -y

RUN rustup target install wasm32-unknown-unknown

RUN cargo install wasm-bindgen-cli
RUN cargo install wasm-opt
RUN cargo install wasm-pack
RUN cargo install just

RUN chmod -R g+w $CARGO_HOME/registry