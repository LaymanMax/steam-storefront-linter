# Multi-stage Dockerfile for Steam Storefront Linter
# 1. Frontend build
FROM node:22-bookworm-slim AS web-builder
WORKDIR /app

# Enable pnpm
RUN corepack enable && corepack prepare pnpm@latest --activate

COPY package.json pnpm-workspace.yaml pnpm-lock.yaml* ./
COPY apps/web/package.json ./apps/web/
RUN pnpm install --frozen-lockfile || pnpm install

COPY apps/web ./apps/web
RUN pnpm --filter web build

# 2. Rust build
FROM rust:1.85-bookworm AS rust-builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev libsqlite3-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
COPY crates ./crates

RUN cargo build --release -p linter-server -p linter-cli

# 3. Runtime image
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries
COPY --from=rust-builder /app/target/release/linter-server /usr/local/bin/linter-server
COPY --from=rust-builder /app/target/release/steam-storefront-linter /usr/local/bin/steam-storefront-linter

# Copy frontend assets
COPY --from=web-builder /app/apps/web/dist /app/apps/web/dist

# Copy fixtures
COPY data/fixtures /app/data/fixtures

# Create data directories
RUN mkdir -p /app/data/processed /app/data/raw

# Initialize sample benchmark database for out-of-the-box functionality
RUN /usr/local/bin/steam-storefront-linter dataset sample -o /app/data/processed/benchmark.db

ENV HOST=0.0.0.0
ENV PORT=3000
ENV BENCHMARK_DB_PATH=/app/data/processed/benchmark.db
ENV WEB_DIST_PATH=/app/apps/web/dist

EXPOSE 3000

CMD ["/usr/local/bin/linter-server"]
