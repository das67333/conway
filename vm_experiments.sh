#!/bin/bash

mv ~/conway/target ~/conway_target_reserved
set -e
rm -rf conway/
git clone https://github.com/das67333/conway/
set +e
mv ~/conway_target_reserved ~/conway/target
set -e

cd conway
sudo apt install -y build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
cargo run --release --bin bench_0e0p | tee ~/out.txt
