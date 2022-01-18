FROM ghcr.io/linuxserver/openssh-server:latest as base

RUN apk update &&\
    apk add python3 &&\
    ln -s /usr/bin/python3 /usr/bin/python &&\
    which python &&\
    python --version
