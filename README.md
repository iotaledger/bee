<h1 align="center">
  <br><img src=".github/Bee.png"></a>
</h1>

<h2 align="center">A framework for building IOTA nodes, clients, and applications in Rust.</h2>

<p align="center">
  <a href="https://discord.iota.org/" style="text-decoration:none;"><img src="https://img.shields.io/badge/Discord-9cf.svg?logo=discord" alt="Discord"></a>
  <a href="https://iota.stackexchange.com/" style="text-decoration:none;"><img src="https://img.shields.io/badge/StackExchange-9cf.svg?logo=stackexchange" alt="StackExchange"></a>
  <a href="https://github.com/iotaledger/bee/blob/master/LICENSE" style="text-decoration:none;"><img src="https://img.shields.io/github/license/iotaledger/bee.svg" alt="Apache 2.0 license"></a>
</p>

<p align="center">
  <img src="https://github.com/iotaledger/bee/workflows/Format/badge.svg">
  <img src="https://github.com/iotaledger/bee/workflows/Audit/badge.svg">
  <img src="https://github.com/iotaledger/bee/workflows/Clippy/badge.svg">
  <img src="https://github.com/iotaledger/bee/workflows/Build/badge.svg">
  <img src="https://github.com/iotaledger/bee/workflows/Test/badge.svg">
  <img src="https://coveralls.io/repos/github/iotaledger/bee/badge.svg?branch=master">
</p>

<p align="center">
  <a href="#about">About</a> ◈
  <a href="#design">Design</a> ◈
  <a href="#supporting-the-project">Supporting the project</a> ◈
  <a href="#joining-the-discussion">Joining the discussion</a>
</p>

---

# About

The IOTA Foundation aims to allow machines of all performance levels to
contribute to the IOTA network, from microcontrollers to phones, web browsers, and servers.

Therefore, Bee is being developed as a modular collection of extendable crates, which expose foreign function interfaces (FFIs) for the next iteration of client libraries.

**Note:** You can find details about future development plans in our [roadmap](https://roadmap.iota.org).

## Design

Bee will be a central reference implementation for the most important
data structures and algorithms. This implementation will be verified during a [Request for Comments](https://github.com/iotaledger/bee-rfcs/) (RFC) process and eventually certified.

By using this approach, we hope that improvements to core components will quickly propagate to all other client libraries, rather than
having to fix each one individually.

**Note:** The Rust programming language was chosen for Bee because of its C/C++ like performance and its strong memory safety guarantees. [Learn more about Rust](https://www.rust-lang.org/).

## Building

You can build a docker image by running the following at the root level of the project
```
docker build -t bee .
```
## Supporting the project

If you want to discuss Bee or have some questions about it, join us on the
[IOTA Discord server](https://discord.iota.org/) in the `#bee-dev` and
`#bee-discussion` channels.

If you want to be a part of development, please see the [contributing guidelines](.github/CONTRIBUTING.md) for information on how to contribute.

## Joining the discussion

If you want to get involved in the community, need help getting started, have any issues related to the repository or just want to discuss blockchain, distributed ledgers, and IoT with other people, feel free to join our [Discord](https://discord.iota.org/).
