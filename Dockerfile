FROM rust:latest AS builder

WORKDIR /

RUN cargo new app

WORKDIR /app

COPY rust-toolchain.toml Cargo.toml Cargo.lock ./

RUN cargo build --release

RUN rm -r src target/release/rating-exchange-bot

COPY migrations ./migrations

COPY src ./src

RUN DATABASE_URL="sqlite://rebot.sqlite3?mode=rwc" cargo build --release

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/rating-exchange-bot /

CMD ["./rating-exchange-bot"]
