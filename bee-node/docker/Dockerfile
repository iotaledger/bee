############################
# Build
############################
FROM rust:1-buster as build

ARG WITH_DASHBOARD=false

LABEL org.label-schema.description="Bee node software to connect to the IOTA and Shimmer networks."
LABEL org.label-schema.name="iotaledger/bee"
LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.vcs-url="https://github.com/iotaledger/bee"
LABEL org.label-schema.usage="https://github.com/iotaledger/bee/blob/mainnet/documentation/docs/getting_started/docker.md"

RUN apt-get update && \
    apt-get install cmake clang -y

WORKDIR /bee
COPY . .

RUN if [ "$WITH_DASHBOARD" = true ] ; then cargo build --profile production --features dashboard --bin bee ; else cargo build --profile production --bin bee ; fi

############################
# Image
############################
# https://console.cloud.google.com/gcr/images/distroless/global/cc-debian11
# using distroless cc "nonroot" image, which includes everything in the base image (glibc, libssl and openssl)
FROM gcr.io/distroless/cc-debian11:nonroot

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

COPY --chown=nonroot:nonroot --from=build /bee/target/production/bee /

# Copy the profiles
COPY --from=build /bee/bee-node/config.chrysalis-comnet.toml .
COPY --from=build /bee/bee-node/config.chrysalis-devnet.toml .
COPY --from=build /bee/bee-node/config.chrysalis-mainnet.toml .

COPY --from=build /bee/bee-node/config.chrysalis-comnet.json .
COPY --from=build /bee/bee-node/config.chrysalis-devnet.json .
COPY --from=build /bee/bee-node/config.chrysalis-mainnet.json .

USER nonroot

ENTRYPOINT ["/bee"]
