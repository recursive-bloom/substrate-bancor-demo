#!/usr/bin/env bash
# This script meant to be run on Unix/Linux based systems

./target/release/node-template purge-chain -y --dev

./target/release/node-template --dev

## https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/extrinsics

