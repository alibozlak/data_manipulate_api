# ---- build stage ----
FROM rust:1.95-slim-bookworm AS builder

WORKDIR /app

# Build the dependency tree against a stub binary first, so that editing src/
# does not invalidate the cached dependency layer.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
# Cargo keys off mtime; without this the stub's artifact would be reused.
RUN touch src/main.rs && cargo build --release

# ---- runtime stage ----
FROM debian:bookworm-slim AS runtime

RUN useradd --system --user-group --no-create-home app

COPY --from=builder /app/target/release/data_manipulate_api /usr/local/bin/data_manipulate_api

USER app

ENV BIND_ADDR=127.0.0.1:3001
EXPOSE 3001

ENTRYPOINT ["/usr/local/bin/data_manipulate_api"]
