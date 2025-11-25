FROM rust:1.91.1-bookworm AS stage

RUN apt update
RUN apt install -y openssl pkg-config libssl-dev


COPY ./backend /app
WORKDIR /app

RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
ENV NVM_DIR=/root/.nvm
ENV SQLX_OFFLINE=true
RUN . /root/.nvm/nvm.sh --no-use && nvm install 24 && nvm use 24 && nvm alias default 24 && node -v && cargo build --release


FROM debian:bookworm-slim

COPY --from=stage /app/target/release/haste-health /haste-health

RUN apt update
RUN apt install -y openssl pkg-config libssl-dev

ENTRYPOINT ["/haste-health"]

EXPOSE 80