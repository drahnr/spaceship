FROM fedora:rawhide

MAINTAINER Bernhard Schuster "jack@ahoi.io"

RUN dnf update -y && dnf search cargo
RUN dnf install -y cargo cmake

RUN mkdir -p /spaceship
WORKDIR /spaceship
ADD . /spaceship/
RUN cargo build --verbose
RUN cargo install
RUN rm -rf src
CMD ["build/Debug/spaceship"]

