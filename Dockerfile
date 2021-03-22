############################
# Build
############################
FROM rust:1 as build

RUN apt-get update && \
    apt-get install cmake clang -y

COPY . /src
WORKDIR /src

RUN cargo build --release --bin bee

############################
# Image
############################
FROM debian:buster-slim

RUN apt-get update && \
    apt-get install openssl -y

RUN rm -rf /var/lib/apt

COPY --from=build /src/target/release/bee /

ENTRYPOINT ["/bee"]
