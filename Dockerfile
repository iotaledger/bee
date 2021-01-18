FROM rust

RUN mkdir bee
WORKDIR /bee
COPY ./Cargo.toml ./Cargo.toml

# bee-api
RUN mkdir ./bee-api
RUN USER=root cargo new --lib bee-api/bee-rest-api
COPY ./bee-api/bee-rest-api/Cargo.toml ./bee-api/bee-rest-api/Cargo.toml

# bee-common
RUN mkdir ./bee-common
RUN USER=root cargo new --lib bee-common/bee-common
COPY ./bee-common/bee-common/Cargo.toml ./bee-common/bee-common/Cargo.toml

# bee-ledger
RUN USER=root cargo new --lib bee-ledger
COPY ./bee-ledger/Cargo.toml ./bee-ledger/Cargo.toml

# bee-message
RUN USER=root cargo new --lib bee-message
COPY ./bee-message/Cargo.toml ./bee-message/Cargo.toml

# bee-network
RUN USER=root cargo new --lib bee-network
COPY ./bee-network/Cargo.toml ./bee-network/Cargo.toml

# bee-node
RUN USER=root cargo new bee-node
COPY ./bee-node/Cargo.toml ./bee-node/Cargo.toml

# bee-peering
RUN USER=root cargo new --lib bee-peering
COPY ./bee-peering/Cargo.toml ./bee-peering/Cargo.toml

# bee-pow
RUN USER=root cargo new --lib bee-pow
COPY ./bee-pow/Cargo.toml ./bee-pow/Cargo.toml

# bee-protocol
RUN USER=root cargo new --lib bee-protocol
COPY ./bee-protocol/Cargo.toml ./bee-protocol/Cargo.toml

# bee-snapshot
RUN USER=root cargo new --lib bee-snapshot
COPY ./bee-snapshot/Cargo.toml ./bee-snapshot/Cargo.toml

# bee-storage
RUN mkdir ./bee-storage
RUN USER=root cargo new --lib bee-storage/bee-storage-rocksdb
COPY ./bee-storage/bee-storage-rocksdb/Cargo.toml ./bee-storage/bee-storage-rocksdb/Cargo.toml

# bee-tangle
RUN USER=root cargo new --lib bee-tangle
COPY ./bee-tangle/Cargo.toml ./bee-tangle/Cargo.toml

# Built only dependencies
RUN cargo build --release

# TODO: copy source files and build again