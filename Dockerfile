FROM rust:1.72.0-bookworm AS build

WORKDIR /app

COPY . .

RUN cargo build --workspace --release

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=build /app/target/release/samply /app/samply

CMD ["/app/samply"]
