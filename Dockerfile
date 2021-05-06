############################
# Prepare dependencies recipe
############################
FROM lukemathwalker/cargo-chef as planner

WORKDIR /bee
COPY . .

RUN cargo chef prepare --recipe-path recipe.json

############################
# Dependency cache
############################
FROM lukemathwalker/cargo-chef as cacher

RUN apt-get update && \
    apt-get install cmake clang -y

WORKDIR /bee
COPY --from=planner /bee/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

############################
# Build
############################
FROM rust:1 as build

WORKDIR /bee
COPY . .

COPY --from=cacher /bee/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME

RUN cargo build --release --bin bee

############################
# Image
############################
FROM debian:buster-slim

RUN apt-get update && \
    apt-get install openssl -y

RUN rm -rf /var/lib/apt

# API
EXPOSE 14265/tcp
# Gossip
EXPOSE 15600/tcp
# MQTT
EXPOSE 1883/tcp
# Dashboard
EXPOSE 8081/tcp

COPY --from=build /bee/target/release/bee /

ENTRYPOINT ["/bee"]
