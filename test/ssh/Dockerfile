FROM ghcr.io/linuxserver/openssh-server:latest as base

RUN apk update &&\
    apk add \
        netcat-openbsd \
        python3 &&\
    ln -s /usr/bin/python3 /usr/bin/python &&\
    which python &&\
    python --version

HEALTHCHECK --interval=3s --retries=20 --timeout=1s \
  CMD nc -z localhost 22
