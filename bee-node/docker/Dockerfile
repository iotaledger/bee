############################
# Build
############################
FROM rust:1-buster as build

ARG WITH_DASHBOARD=false

LABEL org.label-schema.description="Bee node software to connect to the IOTA and Shimmer networks."
LABEL org.label-schema.name="iotaledger/bee"
LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.vcs-url="https://github.com/iotaledger/bee"

RUN apt-get update && \
    apt-get install cmake clang -y

WORKDIR /bee
COPY . .

RUN if [ "$WITH_DASHBOARD" = true ] ; then cargo build --profile production --features dashboard --bin bee ; else cargo build --profile production --bin bee ; fi

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
# Autopeering
EXPOSE 14626/udp

COPY --from=build /bee/target/production/bee /

ENTRYPOINT ["/bee"]
