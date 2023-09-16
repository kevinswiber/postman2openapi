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

RUN if [ `uname -m` = "x86_64" ]; then \
      apt-get install -y libatk-bridge2.0-0 libcups2 libdrm2 libxkbcommon0 libxcomposite1 libxdamage1 libxfixes3 \
            libxrandr2 libgbm1 libpango-1.0-0 libcairo2 libasound2 jq && \
      CHROME_STABLE_DOWNLOADS=$(curl https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json  | jq -r '.channels.Stable.downloads') && \
      CHROME_URL=$(echo $CHROME_STABLE_DOWNLOADS | jq -r '.chrome[] | select(.platform == "linux64").url') && \
      wget -N "$CHROME_URL" -P ~/ && \
      unzip ~/chrome-linux64.zip -d ~/ && \
      mv ~/chrome-linux64 /opt/chrome && \
      ln -s /opt/chrome/chrome /usr/local/bin/chrome && \
      chmod +x /opt/chrome && \
      rm ~/chrome-linux64.zip && \
      CHROMEDRIVER_URL=$(echo $CHROME_STABLE_DOWNLOADS | jq -r '.chromedriver[] | select(.platform == "linux64").url') && \
      wget -N "$CHROMEDRIVER_URL" -P ~/ && \
      unzip ~/chromedriver-linux64.zip -d ~/ && \
      mv ~/chromedriver-linux64 /opt/chromedriver && \
      ln -s /opt/chromedriver/chromedriver /usr/local/bin/chromedriver && \
      chmod +x /opt/chromedriver && \
      rm ~/chromedriver-linux64.zip; fi