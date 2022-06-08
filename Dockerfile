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

RUN rustup update 1.59 && rustup default 1.59

RUN cd /src \
    && sed -i -e "s/openssl.*=.*//" Cargo.toml \
    && RUSTFLAGS="-C target-feature=-crt-static" cargo build --release

FROM alpine:3.16 as tool

RUN apk update && apk add libgcc

COPY --from=base /src/target/release/aim /usr/local/bin

ENTRYPOINT [ "aim" ]
CMD [ "--help" ]
