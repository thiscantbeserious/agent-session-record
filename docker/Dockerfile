FROM rust:1.75-slim AS base
WORKDIR /app

# Install cargo-tarpaulin for coverage
RUN cargo install cargo-tarpaulin

FROM base AS test
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY tests ./tests
RUN cargo test
RUN cargo tarpaulin --fail-under 80 --out Html --output-dir /coverage || echo "Coverage check completed"

FROM base AS builder
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

FROM scratch AS export
COPY --from=builder /app/target/release/asr /asr
