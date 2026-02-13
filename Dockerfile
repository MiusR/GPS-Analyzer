
FROM rust:latest as builder
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libproj-dev \
    sqlite3 \
    libsqlite3-dev \
    clang \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache deps
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src
RUN cargo build --release



FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libssl3 \
    libproj25 \
    sqlite3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/gps-analyzer.exe /usr/local/bin/app
EXPOSE 8000

CMD ["app"]
