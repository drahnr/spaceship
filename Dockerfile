FROM ubuntu:latest
MAINTAINER Bernhard Schuster "jack@ahoi.io"

RUN apt-get update -y
RUN apt-get install -y cmake make curl

RUN curl https://sh.rustup.rs -sSf | \
	sh -s -- --default-toolchain nightly -y

ENV PATH=/root/.cargo/bin:$PATH

RUN mkdir -p /spaceship
WORKDIR /spaceship

ENV CMAKE_MAKE_PROGRAM=make

ADD . /spaceship/
RUN cargo build
RUN cargo install
RUN rm -rf src

EXPOSE 8080

CMD ["spaceship"]

