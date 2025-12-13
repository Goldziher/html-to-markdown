#!/usr/bin/env bash
set -euo pipefail

target="${TARGET:?TARGET is required}"

sudo apt-get update
sudo apt-get install -y pkg-config libssl-dev
case "${target}" in
x86_64-unknown-linux-musl)
	sudo apt-get install -y musl-tools
	;;
aarch64-unknown-linux-gnu)
	sudo apt-get install -y gcc-aarch64-linux-gnu
	;;
esac
