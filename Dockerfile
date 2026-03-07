# Stage 1: Build frontend
FROM node:20-alpine AS frontend
RUN corepack enable && corepack prepare pnpm@latest --activate
WORKDIR /build
COPY web/package.json web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY web/ ./
RUN pnpm build

# Stage 2: Build backend
FROM rust:latest AS backend
WORKDIR /build
# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/common/Cargo.toml crates/common/Cargo.toml
COPY crates/auth/Cargo.toml crates/auth/Cargo.toml
COPY crates/storage/Cargo.toml crates/storage/Cargo.toml
COPY crates/core/Cargo.toml crates/core/Cargo.toml
COPY crates/api/Cargo.toml crates/api/Cargo.toml
COPY crates/webdav/Cargo.toml crates/webdav/Cargo.toml
COPY migration/Cargo.toml migration/Cargo.toml
# Create dummy sources so cargo can resolve the workspace
RUN mkdir -p src && echo "fn main() {}" > src/main.rs \
    && mkdir -p crates/common/src && echo "" > crates/common/src/lib.rs \
    && mkdir -p crates/auth/src && echo "" > crates/auth/src/lib.rs \
    && mkdir -p crates/storage/src && echo "" > crates/storage/src/lib.rs \
    && mkdir -p crates/core/src && echo "" > crates/core/src/lib.rs \
    && mkdir -p crates/api/src && echo "" > crates/api/src/lib.rs \
    && mkdir -p crates/webdav/src && echo "" > crates/webdav/src/lib.rs \
    && mkdir -p migration/src && echo "" > migration/src/lib.rs
RUN cargo build --release 2>/dev/null || true
# Copy real sources and build
COPY src/ src/
COPY crates/ crates/
COPY migration/ migration/
RUN touch src/main.rs crates/*/src/lib.rs migration/src/lib.rs \
    && cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend /build/target/release/mediadrive-pro ./
COPY --from=frontend /build/dist/ ./web/dist/
COPY config.example.toml ./config.toml

RUN mkdir -p data uploads

EXPOSE 8080
CMD ["./mediadrive-pro"]
