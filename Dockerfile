FROM rust:1.96.0

WORKDIR /app

RUN app update && apt install lld clang -y

COPY . .

RUN cargo build --release

ENTRYPOINT ["./target/release/zero2prod-email-newsletter"]