FROM rust:slim-bullseye as builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libc-dev \
    libssl-dev \
    libsensors-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
RUN cargo init
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release --features http,sensors
RUN cargo clean -p minmon

COPY ./src ./src
RUN cargo install --features http,sensors --path .


FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    libsensors5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/minmon /usr/local/bin

ENTRYPOINT ["/usr/local/bin/minmon"]
CMD ["/etc/minmon.toml"]
