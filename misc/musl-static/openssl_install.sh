#!/bin/bash

set -eu

apt install musl-tools
wget https://github.com/openssl/openssl/archive/OpenSSL_1_1_1f.tar.gz
tar zxvf OpenSSL_1_1_1f.tar.gz
cd openssl-OpenSSL_1_1_1f
mkdir $(pwd)/musl
./Configure no-shared no-async --prefix=$(pwd)/musl --openssldir=$(pwd)/musl/ssl --cross-compile-prefix=aarch64-linux-gnu- linux-aarch64
make
make install

export PKG_CONFIG_ALLOW_CROSS=1
export OPENSSL_STATIC=true
export OPENSSL_DIR=/home/imeoer/musl

env CC=aarch64-linux-gnu-gcc RUSTFLAGS="-C linker=aarch64-linux-gnu-gcc" cargo build --target aarch64-unknown-linux-musl --features=fusedev --release --target-dir target-fusedev --bin nydusd
