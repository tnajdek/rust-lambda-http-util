#!/bin/bash

cargo build --features with-lambda --release --target x86_64-unknown-linux-musl
cp ./target/x86_64-unknown-linux-musl/release/rust-lambda-http-util ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
