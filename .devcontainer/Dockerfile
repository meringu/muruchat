FROM mcr.microsoft.com/vscode/devcontainers/rust:dev-bullseye

# Install Trunk CLI
RUN cargo install trunk

# Add WASM target
RUN rustup target add wasm32-unknown-unknown

# Install Tailwind CSS
RUN curl -fsLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64 && \
    chmod +x tailwindcss-linux-x64 && \
    mv tailwindcss-linux-x64 /usr/local/bin/tailwindcss

# Install Wrangler
RUN apt update && \
    curl -sL https://deb.nodesource.com/setup_lts.x | sudo bash - && \
    apt install -y nodejs
RUN npm install --location=global wrangler@2
