FROM rust:slim-bookworm as builder

WORKDIR /usr/src/gitea-plugin-rs
COPY . .

RUN cargo build --release --bin webhook-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/gitea-plugin-rs/target/release/webhook-server /usr/local/bin/webhook-server

ENV HOST=0.0.0.0
ENV PORT=3000

EXPOSE 3000

CMD ["webhook-server"]
