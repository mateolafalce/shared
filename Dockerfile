FROM rust:1.90.0-bookworm AS builder

WORKDIR /app

COPY Cargo.toml ./
COPY src ./src
COPY static ./static

RUN cargo build --release

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/shared .

EXPOSE 3000

CMD ["./shared"]