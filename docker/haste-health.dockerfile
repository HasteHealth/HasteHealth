FROM rust:1.98.0-bookworm AS builder

RUN apt update && apt install -y openssl pkg-config libssl-dev && apt clean

RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
ENV NVM_DIR=/root/.nvm

ENV SQLX_OFFLINE=true

WORKDIR /build
COPY ./artifacts ./artifacts
COPY ./backend ./backend
WORKDIR /build/backend
RUN . /root/.nvm/nvm.sh --no-use && nvm install 24 && nvm use 24 && nvm alias default 24 && node -v && cargo build --locked --release


FROM debian:bookworm-slim

COPY --from=builder /build/backend/target/release/haste-health /haste-health

RUN apt-get update && apt-get upgrade -y && apt-get install -y ca-certificates openssl pkg-config libssl-dev && apt-get clean && rm -rf /var/lib/apt/lists/*

ENTRYPOINT ["/haste-health"]