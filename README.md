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
