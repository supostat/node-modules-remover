# Build stage
FROM rust:1.83-bookworm AS builder

WORKDIR /app

# Install dependencies for cross-compilation
RUN apt-get update && apt-get install -y \
    musl-tools \
    musl-dev \
    && rm -rf /var/lib/apt/lists/*

# Add musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual binary
RUN touch src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage - minimal image
FROM alpine:3.19 AS runtime

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/nm-remover /usr/local/bin/nm-remover

# Create a non-root user
RUN adduser -D -u 1000 appuser
USER appuser

WORKDIR /workspace

ENTRYPOINT ["nm-remover"]
CMD ["--help"]

# Development stage
FROM rust:1.83-bookworm AS development

WORKDIR /app

# Install useful development tools
RUN apt-get update && apt-get install -y \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-watch for hot reloading
RUN cargo install cargo-watch

# Copy the project
COPY . .

# Default command for development
CMD ["cargo", "watch", "-x", "run"]
