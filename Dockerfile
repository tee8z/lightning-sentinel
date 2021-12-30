####################################################################################################
## Builder
####################################################################################################
FROM rust:alpine3.14 AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apk upgrade --update-cache --available && \
    apk add --no-cache \
            libressl-dev \
            musl-dev \
            libffi-dev \
            make \
            automake \
            autoconf

RUN update-ca-certificates

WORKDIR /lightning-sentinel

COPY ./ .

RUN RUSTFLAGS='-C target-feature=+crt-static -C link-self-contained=yes' cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
#FROM scratch

# Import from builder.
#COPY --from=builder /etc/passwd /etc/passwd
#COPY --from=builder /etc/group /etc/group

#WORKDIR /lightning-sentinel

# Copy our build
#COPY --from=builder /lightning-sentinel/target/x86_64-unknown-linux-musl/release/lightning-sentinel ./

# Use an unprivileged user.
#USER lightning-sentinel:lightning-sentinel

#CMD ["./lightning-sentinel"]