FROM rust:1.81-slim-bullseye AS build

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev

WORKDIR /usr/src/app

COPY src src
COPY Cargo.toml .
COPY Cargo.lock .

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /usr/local/bin

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser \
    && chown -R appuser .
USER appuser

COPY --from=build /usr/src/app/target/release/lnrecc .

CMD ["lnrecc"]