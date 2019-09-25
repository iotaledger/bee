# The IOTA Bee framework

Bee is a Rust framework to build IOTA nodes and tools that interact with the IOTA network. The final product is envisioned to be a highly modular collection of foundational crates that can be mixed and matched, and extended upon.

Bee may be seen as an effort to streamline the IOTA Foundation's different libraries into one fundamental collection of Rust crates. These will expose foreign function interfaces (FFIs) to be used in more high level languages, and form the basis for the next iteration of client APIs.

## Outline

+ [Motivation]
+ [Milestones]
+ [Contributing]
+ [Why Rust?]

## Motivation
[Motivation]: #motivation

Our primary motivation is to make IOTA an exemplary open source project, with
great documentation, clear examples of increasing complexity, and well-tested
code. Entry into the IOTA developer ecosystem should be as straight forward as
possible, so that people can start contributing, extending, and building their
ideas on top of the IOTA network using the Bee framework. As such, we want to
be ready for development driven by our own dev and research teams, as well as
novel ideas coming out of our community.

Bee will provide one central reference implementation of the most important
data structures and algorithms, which will be verified and eventually
certified. By this approach, we hope that improvements in any core components
will quickly propagate to all other libraries building on them, rather than
having to fix each single implementation we encounter.

The IOTA Foundation aims to allow machines of all performance levels to
contribute to the IOTA network, from microcontrollers and single-board
computers, to phones, web browsers, desktop machines and servers. We therefore
want to implement Bee's constituent libraries from the ground up, to allow the
import of each library on its own, with each library having as few dependencies
as possible. Having microcontrollers in mind, we want these libraries to not
rely on a runtime — this includes language runtimes such as garbage
collection, but also OS integration (which, in the case of Rust, is provided by
its `libstd` standard library). 

## Milestones
[Milestones]: #milestones

The Bee framework was primarily created to support the implementation of the
IOTA [Coordicide](https://coordicide.iota.org/). As previously stated, on the
path to Coordicide we aim to make this framework as open, vetted and usable as
possible, by supporting the creation of tools, clients, nodes…

Our proposed milestones are:

1. **Fundamental crates**: Specification and implementation of the Bee fundamental crates `bee-trinary`, `bee-model`, and `bee-crypto`. These are the essential bricks for IOTA development.
2. **FFI**: Foreign Function Interface to make the Bee crates available to other languages. The first project relying on Bee crates through FFI will be Trinity v2.
    + We will also investigate in how far WebAssembly, `wasm`, might be an intermediate milestone.
3. **Rust IRI**: Specification and implementation of the node-specific crates `bee-network`, `bee-tangle`, `bee-api`, `bee-consensus`, and `bee-gossip`. To demonstrate the modularity and robustness of the Bee framework, a node for the current mainnet will be implemented. Some of these crates will be repurposed for the Coordicide node.
4. **Coordicide**: Specification and implementation of new Coordicide node-specific crates once research specifications are delivered.

## Contributing
[Contributing]: #contributing

If you want to discuss Bee or have some questions about it, join us on the
[IOTA Discord server](https://discord.iota.org/) in the `#bee-dev` and
`#bee-discussion` channels.

If you want to contribute bug reports and pull requests or suggest new
features, please see
[`bee/CONTRIBUTING.md`](./CONTRIBUTING.md)
for guidelines on how to contribute. Please note that we have a Request for
Comments (RFC) process in place to propose, discuss, and vote on new features
entering the Bee framework. You can find more information at
[`iotaledger/bee-rfcs`](https://github.com/iotaledger/bee-rfcs/).

## Why Rust?
[Why Rust?]: #why-rust

The IOTA Foundation aims to allow machines of all performance levels to
contribute to the IOTA network: from microcontrollers to phones and servers.
Some of these systems will have minimal resources and lack an operating system.
Microcontrollers aside, we also aim for energy efficiency, high performance,
and a great user and developer experience.

From the available choice of languages, we felt that Rust was the best fit.
Rust being a system programming language, it operates at a high level of
performance while providing strong memory safety guarantees. Together with its
well-integrated modern tooling, this helps us iterate on our designs faster
while being more confident that our code does what we intend it to do (without
accessing uninitialized memory, double free, use after free, null pointer
dereference, and buffer overflows that are all security concerns).
