# A Framework For Building IOTA Nodes, Clients, and Applications in Rust

![Badge](https://github.com/iotaledger/bee/blob/chrysalis-pt-2/.github/Bee.png?raw=true "Badge")

[![Discord](https://img.shields.io/badge/Discord-9cf.svg?logo=discord "Discord")](https://discord.iota.org/)
[![StackExchange](https://img.shields.io/badge/StackExchange-9cf.svg?logo=stackexchange "StackExchange")](https://iota.stackexchange.com/)
[![Apache 2.0 license](https://img.shields.io/github/license/iotaledger/bee.svg "Apache 2.0 license")](https://github.com/iotaledger/bee/blob/master/LICENSE)

![Format Badge](https://github.com/iotaledger/bee/workflows/Format/badge.svg "Format Badge")
![Audit Badge](https://github.com/iotaledger/bee/workflows/Audit/badge.svg "Audit Badge")
![Clippy Badge](https://github.com/iotaledger/bee/workflows/Clippy/badge.svg "Clippy Badge")
![BuildBadge](https://github.com/iotaledger/bee/workflows/Build/badge.svg "Build Badge")
![Test Badge](https://github.com/iotaledger/bee/workflows/Test/badge.svg "Test Badge")
![Coverage Badge](https://coveralls.io/repos/github/iotaledger/bee/badge.svg?branch=dev "Coverage Badge")


# About

The IOTA Foundation aims to allow machines of all performance levels to contribute to the IOTA network, from microcontrollers to phones, web browsers, and servers.

Therefore, we are developing Bee as a modular collection of extendable crates, which expose foreign function interfaces (FFIs) for the next iteration of client libraries.

::info
You can find details about future development plans in our [roadmap](https://roadmap.iota.org).
:::

## Design

Bee will be a central reference implementation for the most important
data structures and algorithms. This implementation will be verified during a [Request for Comments](https://github.com/iotaledger/bee-rfcs/) (RFC) process, and eventually certified.

By using this approach, we hope that improvements to core components will quickly propagate to all other client libraries, rather than
having to fix each one individually.

:::info
We have chosen the Rust programming language for Bee because of its C/C++ like performance, and its strong memory safety guarantees. [Learn more about Rust](https://www.rust-lang.org/).
:::

## Supporting the Project

If you want to discuss Bee, or have some questions about it, join us on the
[IOTA Discord server](https://discord.iota.org/) in the `#bee-dev` and
`#bee-discussion` channels.

If you want to be a part of development, please see the [contributing guidelines](contribute/contribute.md) for information on how to contribute.

## Joining the Discussion

If you want to get involved in the community, need help getting started, have any issues related to the repository, or just want to discuss blockchain, distributed ledgers, and IoT with other people, feel free to join our [Discord](https://discord.iota.org/).
