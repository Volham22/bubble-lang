FROM rust:1.71-bullseye

RUN rustup component add rustfmt
RUN rustup component add clippy

# Install llvm15
RUN wget https://apt.llvm.org/llvm.sh
RUN chmod +x llvm.sh
RUN apt update -y
RUN apt install -y lsb-release wget software-properties-common gnupg
RUN ./llvm.sh 16 all
RUN ln -s /usr/bin/clang-16 /usr/bin/clang
# Ugly hack to make ld  hardcoded crt*.o object files work with debian based system
RUN ln -s /usr/lib/x86_64-linux-gnu/ /usr/lib64

RUN mkdir /app

WORKDIR /app
