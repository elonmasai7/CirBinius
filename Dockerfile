FROM rust:1.91-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p cirbinius-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /opt/cirbinius
COPY --from=builder /app/target/release/cirbinius-api /usr/local/bin/cirbinius-api
EXPOSE 8080
CMD ["/usr/local/bin/cirbinius-api"]
