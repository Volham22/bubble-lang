FROM rust:1.71-alpine

RUN apk add llvm15
RUN rustup component add rustfmt
RUN rustup component add clippy
RUN mkdir /app

WORKDIR /app
