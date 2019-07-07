# Bee - IOTA for the IoT

## Motivation

Providing data integrity and support for machine-to-machine micropayments is IOTA's vision in order to become the backbone protocol for Internet-of-Things (IoT). While the IoT will keep emerging and transforming itself over the next few years, IOTA has already brought its protocol to reality.

The current IOTA mainnet is globally distributed network of nodes, each running a node software: the [IOTA Reference Implementation (IRI)](https://github.com/iotaledger/iri). This node software has been developed for deployment in today's global cloud-based Internet. However, the current Internet has completely different properties and requirements than the future IoT. To deliver on it's promise, IOTA must provide a solution that tackles the various difficulties of this future environment.

## Vision

Bee is supposed to deliver on a varity of qualities with different priorities. Some are orthogonal, others are tricky to combine and must be carefully balanced out:

Quality | Description
-- | --
Performance | Scalability and security of the Tangle depend on how many transactions can be processed. Resources like bandwidth, memory and computing power are limited. To use them to the fullest, performance is a major concern. The more efficient a node operates, the more transactions it can process and the more scalable and secure the network becomes.
Light-Weightness | When it comes to computing, we think about servers, PC's and mobile devices. The IoT seeks to integrate countless tiny  devices into the network to take full advantage of available resources. Therefore it is necessary to grant even rather constrained devices access to the Tangle. The bee core (minimum required software to run a node) must be as slim as possible and be able to operate even if with limited resources.
Interoperability | The IoT will be no less heterogenous than today's Internet. Not only devices will differ in hardware but there will be many physical means and standards for data transmission (optical, bluetooth, RFI, ...). Building on top of this infrastructure requires abstracting from individual details yet still considering these for maximum efficiency.
Extendibility |  IOTA seeks to become the standard protocol for applications developed for the IoT. These applications must be able to connect to build on top of this protocol by extending the bee core with use-case specific functionality.
Modularity | Building a protocol for the IoT is a huge undertaking. To reduce bugs during and increase speed of the development process, software quality is a major concern. A modular architecture is the best step towards a maintainable, flexible and long-living code base.

## Implementation

### Design Decisions

Category | Decision | Description
-- | -- | --
Programming Language | Rust | A low level programming language
Software Architecture | EEE | Environment-Entity-Effect: an event driven software architecture proposed by the IOTA Foundation

### Solutions

Solution | Supported Quality | Description
-- | -- | --
Rust | Performance | Due to its low level, Rust performance is on par with C. Rust employs additional safety-mechanisms to reduce the amount of bugs that are common with C (such as memory corruption or concurrency issues). However, these checks happen during compile-time and have therefore rarely consequences on the performance. C is a sub-set of Rust and can therefore still be employed if necessary.
Rust | Light-Weightness | Once compiled, Rust is as light-weight as C for the same reasons.
Rust | Interoperability | Rust is quite portable and can be compiled for individual processors.
EEE | Light-Weightness | EEE strongly decouples components in such a way that new components can often be added without affecting the existing ones. Therefore the bee core can be minimal (a gossip protocol) with extending features (e.g. consensus) being added only if required.
EEE | Interoperability | Due to the decoupling provided by EEE, component implementations can be easily swapped out or customized without breaking other components.
EEE | Extendibility | With EEE, components communicate only indirectly via environments with each other. Therefore, they are not necessarily aware of which components are participating in an environment but only about the purpose of the environment. Therefore extensions can be smoothly added that plug themselves into the existing communication infrastructure. Since any relevant communication path will be done via EEE, there will be little limits of what they can technically extend.
EEE | Modularity | EEE enforces a strong decoupling by nature.

## Crates

In Rust, a crate provides a library for a clear responsiblity. Every major component in bee will be developed in a seperate crate for modularity reasons. Required crates can then be plugged together to form an executable bee node.

Crate | Responsibility
-- | --
EEE | Provides the architectural framework for inter-crate and a lot of intra-crate communication.
Main | Executes a regular run of a bee node by spinning up and maintaining a single bee instance.
Core | Provides control over a bee node instance and bundles bee components together.
Transport | Data transmission to neighbor bees. Implemented protocol is hidden.
Node | Gossip: logic for receiving and broadcasting transactions.
Trinary | Library for trinary operations and conversions.
Hash | Provides the protocol's default hash function (to derive transaction hashes etc.).
Compression | Library to compress/decompress data before/after transmission to neighbors.
Tangle | Provides data structure to model, store/access and link transactions.
