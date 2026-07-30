FROM rust:1.97.1-bookworm AS chef

RUN apt update && apt install -y openssl pkg-config libssl-dev && apt clean
RUN cargo install cargo-chef --locked

RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
ENV NVM_DIR=/root/.nvm


FROM chef AS planner

COPY ./backend .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder

ENV SQLX_OFFLINE=true

COPY --from=planner recipe.json recipe.json
RUN . /root/.nvm/nvm.sh --no-use && nvm install 24 && nvm use 24 && nvm alias default 24 && node -v && cargo chef cook --release --recipe-path recipe.json

COPY ./backend .
RUN . /root/.nvm/nvm.sh --no-use && nvm use default && cargo build --locked --release


FROM debian:bookworm-slim

COPY --from=builder /target/release/haste-health /haste-health

RUN apt update && apt install -y ca-certificates openssl pkg-config libssl-dev && apt clean

ENTRYPOINT ["/haste-health"]
