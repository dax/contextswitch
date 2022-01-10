#!/bin/bash

sudo apt-get update -y
sudo apt-get install -y --no-install-recommends openssl taskwarrior

out_type="$1"
version="$2"
args="$3"
tar_file="cargo-tarpaulin-${version}-travis.tar.gz"

wget "https://github.com/xd009642/tarpaulin/releases/download/${version}/${tar_file}"
tar zxvf "$tar_file"
chmod +x cargo-tarpaulin

exec env RUST_LOG=debug ./cargo-tarpaulin tarpaulin --ignore-tests -o "$out_type" $args
