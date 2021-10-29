#!/bin/bash

cross build --release --target x86_64-unknown-linux-musl && \
zip -j rust.zip ./target/x86_64-unknown-linux-musl/release/bootstrap && \
sam deploy --guided --profile test