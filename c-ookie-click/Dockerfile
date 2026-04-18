# stolen from https://github.com/fly-apps/hello-rust/blob/main/Dockerfile

FROM rust:latest AS builder

WORKDIR /usr/src/app
COPY . .
RUN apt update && apt install -y mold
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/c-ookie-click ./c-ookie-click

# Runtime image
FROM debian:stable-slim
RUN apt update && apt install -y curl

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/src/app/c-ookie-click /app/c-ookie-click

# Run the app
CMD ./c-ookie-click
