FROM clux/muslrust:1.49.0

RUN rustup component add clippy
RUN rustup component add rustfmt
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /nydus-rs

CMD make static-release
