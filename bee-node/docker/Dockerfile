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

ENV NVM_DIR /opt/nvm
ENV NODE_VERSION 14.19.0

RUN if [ "$WITH_DASHBOARD" = true ] ; then \ 
    mkdir $NVM_DIR \
    &&curl https://raw.githubusercontent.com/creationix/nvm/v0.39.1/install.sh | bash \
    && . $NVM_DIR/nvm.sh \
    && nvm install $NODE_VERSION \
    && nvm alias default $NODE_VERSION \
    && nvm use default \
    && git submodule update --init --recursive \
    && cd bee-node/src/plugins/dashboard/frontend \
    && npm install \
    && npm run build-bee \
    ; fi

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

# Copy the profiles
COPY --from=build /bee/bee-node/config.chrysalis-comnet.toml .
COPY --from=build /bee/bee-node/config.chrysalis-devnet.toml .
COPY --from=build /bee/bee-node/config.chrysalis-mainnet.toml .

ENTRYPOINT ["/bee"]
