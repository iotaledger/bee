---
description: In this section, you will find an overview of all the crates that make up Bee.
image: /img/logo/bee_logo.png
keywords:
- troubleshooting
- rust
- crate
- reference
---
# Overview

In this section, you will find references to all of the crates that make up Bee.  

## bee-api

The default REST API implementation for the IOTA Bee node software.

## bee-block

Implementation of [RFC: Block](https://github.com/GalRogozinski/protocol-rfcs/blob/block/text/0017-block/0017-block.md).

## bee-ledger

All types and features required to compute and maintain the ledger state.

## bee-gossip

Networking functionality and types for nodes and clients participating in the IOTA protocol built on top of `libp2p`.

## bee-pow

Provides Proof of Work utilities for the IOTA protocol.

## bee-protocol

All types and workers enabling the IOTA protocol.

## bee-runtime

Runtime components and utilities for the bee framework.

## bee-storage

#### bee-storage

A general purpose storage backend crate with a key:value abstraction API.

#### bee-storage-rocksdb

A bee-storage implementation for the [RocksDB](https://rocksdb.org/) backend.

#### bee-storage-sled

A bee-storage implementation for the [Sled](https://dbdb.io/db/sled) backend.


#### bee-storage-test

A crate to test storage implementation generically.
