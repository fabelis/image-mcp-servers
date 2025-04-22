# Base Rust image for building
FROM rust:latest AS builder

# Set working directory
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Create a new empty Rust project to cache dependencies
RUN cargo new --bin image-mcp-servers
WORKDIR /app/image-mcp-servers

# Copy only the dependency files (to leverage Docker layer caching)
COPY Cargo.toml Cargo.lock ./

# Fetch dependencies without building the project
RUN cargo fetch

# Copy actual source files after caching dependencies
COPY src ./src

# Set build features as an argument
ARG BUILD_FEATURES=""
RUN cargo build --release --features "$BUILD_FEATURES"

# Final minimal runtime image
FROM debian:bookworm-slim

# Install required system libraries
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy only the final executable, skipping build artifacts
COPY --from=builder /app/image-mcp-servers/target/release/image-mcp-servers /usr/local/bin/image-mcp-servers

EXPOSE ${MCP_PORT}

CMD ["image-mcp-servers"]