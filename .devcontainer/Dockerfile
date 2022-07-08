FROM mcr.microsoft.com/vscode/devcontainers/rust:dev-bullseye

# Install Trunk CLI
RUN cargo install trunk

# Add WASM target
RUN rustup target add wasm32-unknown-unknown

# Install Node.js LTS
RUN apt update
RUN curl -sL https://deb.nodesource.com/setup_lts.x | sudo bash -
RUN apt install -y nodejs

# Install Tailwind CSS
RUN npm install --location=global tailwindcss@3

# Install Wrangler
RUN npm install --location=global wrangler@2