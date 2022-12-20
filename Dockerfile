FROM rust:alpine as builder

RUN apk add --no-cache musl-dev openssl-dev lm-sensors-dev

WORKDIR /app
RUN cargo init
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release --features sensors
RUN cargo clean -p minmon

COPY ./src ./src
RUN cargo install --path .


FROM alpine

RUN apk add --no-cache openssl lm-sensors-libs

COPY --from=builder /usr/local/cargo/bin/minmon /usr/local/bin

ENTRYPOINT ["/usr/local/bin/minmon"]
CMD ["/etc/minmon.toml"]
