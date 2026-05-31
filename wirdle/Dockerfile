FROM rust:1.96-bookworm AS build

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static
RUN cargo build --release --bin wordle-server

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=build /app/target/release/wordle-server /usr/local/bin/wordle-server
COPY wordle-data ./wordle-data

ENV WORDLE_DATA_DIR=/app/wordle-data
ENV PORT=10000

EXPOSE 10000
CMD ["wordle-server"]
