FROM fedora:latest
MAINTAINER Bernhard Schuster "jack@ahoi.io"

RUN dnf update -y

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain nightly -y

ENV PATH=/root/.cargo/bin:$PATH

RUN mkdir -p /spaceship
WORKDIR /spaceship

RUN dnf install -y cmake gcc llvm clang
RUN dnf install -y make
ENV CMAKE_MAKE_PROGRAM=make

ADD . /spaceship/
RUN cargo build --verbose
RUN cargo install
RUN rm -rf src
CMD ["build/Debug/spaceship"]

