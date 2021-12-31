####################################################################################################
## Builder
####################################################################################################
FROM rust:1.57-bullseye AS builder

RUN rustup target add x86_64-unknown-linux-gnu

RUN apt-get update && \
    apt-get install -y \
            libssl-dev \
            musl-dev \
            make

RUN update-ca-certificates

WORKDIR /lightning-sentinel

COPY ./ .


