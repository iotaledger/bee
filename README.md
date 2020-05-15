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
  <a href="#about">About</a> ◈
  <a href="#design">Design</a> ◈
  <a href="#supporting-the-project">Supporting the project</a> ◈
  <a href="#joining-the-discussion">Joining the discussion</a> 
</p>

---

# About

The IOTA Foundation aims to allow machines of all performance levels to
contribute to the IOTA network, from microcontrollers and single-board
computers, to phones, web browsers, desktop machines, and servers.

Therefore, Bee is being developed as a modular collection of extendable crates, which expose foreign function interfaces (FFIs) for the next iteration of client libraries.

**Note:** You can find details about future development plans in our [roadmap](https://roadmap.iota.org).

## Design

Bee will provide one central reference implementation of the most important
data structures and algorithms, which will be verified and eventually
certified.

By using this approach, we hope that improvements in any core components will quickly propagate to all other client libraries, rather than
having to fix each one individually.

## Supporting the project

If you want to discuss Bee or have some questions about it, join us on the
[IOTA Discord server](https://discord.iota.org/) in the `#bee-dev` and
`#bee-discussion` channels.

If you want to be a part of development, please see the [contributing guidelines](.github/CONTRIBUTING.md) for information on how to contribute.

**Note:** We have a Request for Comments (RFC) process in place to propose, discuss, and vote on new features for the Bee framework. You can find more information at [`iotaledger/bee-rfcs`](https://github.com/iotaledger/bee-rfcs/).

## Joining the discussion

If you want to get involved in the community, need help getting started, have any issues related to the repository or just want to discuss blockchain, distributed ledgers, and IoT with other people, feel free to join our [Discord](https://discord.iota.org/).
