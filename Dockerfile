FROM rust:alpine3.16 as base
RUN apk update \
    && apk add \
        git \
        gcc \
        g++ \
        openssl \
        openssl-dev \
        pkgconfig

COPY . /src

RUN rustup update 1.85.0 && rustup default 1.85.0

RUN cd /src \
    && sed -i -e "s/openssl.*=.*//" Cargo.toml \
    && RUSTFLAGS="-C target-feature=-crt-static" cargo build --release

FROM alpine:3.23 as tool

RUN apk update && \
    apk add \
      libgcc \
      openssl

COPY --from=base /src/target/release/aim /usr/local/bin

ENTRYPOINT [ "aim" ]
CMD [ "--help" ]
