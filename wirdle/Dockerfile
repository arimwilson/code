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

# Precompute the first-turn statistics into the image, so instance cold starts
# load them (milliseconds) instead of running the full sweep (~8 CPU-seconds,
# far longer on throttled instances). The server validates the cache against
# the word data at startup and recomputes if it is stale, so a bad or missing
# cache degrades to slower startup, never wrong results.
RUN wordle-server --warm-cache

ENV PORT=10000

EXPOSE 10000
CMD ["wordle-server"]
