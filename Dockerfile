FROM rust:alpine3.14 as base
RUN apk update \
    && apk add \
        git \
        gcc \
        g++ \
        openssl \
        openssl-dev \
        pkgconfig

RUN mkdir -p /src && \
    cd /src && \
    git clone --depth 1 https://github.com/mihaigalos/aim && \
    cd aim && \
    RUSTFLAGS="-C target-feature=-crt-static" cargo build --release

FROM alpine:3.14 as tool

RUN apk update && apk add libgcc

COPY --from=base /src/aim/target/release/aim /usr/local/bin

WORKDIR /src

ENTRYPOINT [ "aim" ]
CMD [ "--help" ]
