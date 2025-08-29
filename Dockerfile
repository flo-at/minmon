FROM rust:slim-bookworm AS builder
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN apt-get update && apt-get install --no-install-recommends --no-install-suggests -y \
    pkg-config \
    libc-dev \
    libssl-dev \
    libsensors-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
RUN cargo init
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release --features docker,http,sensors,smtp
RUN cargo clean -p minmon

COPY ./src ./src
RUN cargo install --features docker,http,sensors,smtp --path .


FROM debian:bookworm-slim

RUN apt-get update && apt-get install --no-install-recommends --no-install-suggests -y \
    ca-certificates \
    openssl \
    libsensors5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/minmon /usr/local/bin

ENTRYPOINT ["/usr/local/bin/minmon"]
CMD ["/etc/minmon.toml"]
