FROM rust:latest AS builder

WORKDIR /

RUN cargo new app

WORKDIR /app

COPY rust-toolchain.toml Cargo.toml Cargo.lock ./

RUN cargo build --release && rm src/main.rs

COPY . /app

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/rating-exchange-bot /

CMD ["./rating-exchange-bot"]
