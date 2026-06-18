FROM rust:1.75-bookworm

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    pkg-config \
    libssl-dev \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js 20 (for VS Code extension build)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Set up workspace
WORKDIR /workspace

# Copy source and build BorrowScope binaries
COPY . /workspace/borrowscope-src
RUN cd /workspace/borrowscope-src \
    && cargo build -p borrowscope-lsp --release \
    && cargo build -p borrowscope-analyzer --release \
    && cp target/release/borrowscope-lsp /usr/local/bin/ \
    && cp target/release/borrowscope-analyzer /usr/local/bin/ \
    && rm -rf target

# Build and package the VS Code extension
RUN cd /workspace/borrowscope-src/borrowscope-vscode \
    && npm install \
    && npm run build

# Clean up source (user will mount their own project)
RUN rm -rf /workspace/borrowscope-src

# Verify binaries are accessible
RUN borrowscope-lsp --version && borrowscope-analyzer --version
