FROM rustlang/rust:nightly-slim

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && mkdir bin && cp target/release/suikei_rs bin/suikei_rs

ENV PORT=8080

ENTRYPOINT ["/app/bin/suikei_rs"]