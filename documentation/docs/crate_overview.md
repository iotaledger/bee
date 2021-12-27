---
description: In this section you will find an overview of all the crates that make up Bee.
image: /img/logo/bee_logo.png
keywords:
- troubleshooting
- rust
- crate
---
# Overview

In this section you will find an overview of all the crates that make up Bee.  


## bee-api

The default REST API implementation for the IOTA Bee node software.

## bee-common

### bee-common

Common utilities used across the bee framework.

### bee-common-derive

Derive macros for the `bee-common` crate.

## bee-crypto

TO-DO

## bee-ledger

All types and features required to compute and maintain the ledger state.

## bee-message

Implementation of [RFC: Message](https://github.com/GalRogozinski/protocol-rfcs/blob/message/text/0017-message/0017-message.md).

## bee-gossip

Networking functionality and types for nodes and clients participating in the IOTA protocol built on top of `libp2p`.

## bee-pow

Provides Proof of Work utilities for the IOTA protocol.

## bee-protocol

All types and workers enabling the IOTA protocol.

## bee-runtime

Runtime components and utilities for the bee framework.

## bee-signing

IOTA signing primitives.

## bee-storage

#### bee-storage

A general purpose storage backend crate with a key:value abstraction API.

#### bee-storage-rocksdb

A bee-storage implementation for the [RocksDB](https://rocksdb.org/) backend.

#### bee-storage-sled

A bee-storage implementation for the [Sled](https://dbdb.io/db/sled) backend.


#### bee-storage-test

A crate to test storage implementation generically.



