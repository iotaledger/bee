# Getting Started

Running a node is the best way to use IOTA. By doing so, you have direct access to the Tangle instead of having to
connect to and trust someone else's node. Additionally, you help the IOTA network to become more distributed and resilient.

The node software is the backbone of the IOTA network. For an overview of tasks a node is responsible for, please
see [Node 101](./nodes_101.md).

To make sure that your device meets the minimum security requirements for running a node, please
see [Security 101](./security_101.md).

## Recommended Requirements

To handle a potential high rate of messages per second, nodes need enough computational power to run reliably, and
should have following minimum specs:

- 4 cores or 4 vCPU
- 8 GB RAM
- SSD storage
- A public IP address

The amount of storage you need will depend on whether and how often you plan on pruning old data from your local
database.

Bee exposes different functionality on different ports:

- 15600 TCP - Gossip protocol port
- 14265 TCP - REST HTTP API port (optional)
- 8081 TCP - Dashboard (optional)

The mentioned ports are important for flawless node operation. The REST HTTP API port is optional and is only needed if
you want to offer access to your node's API. All ports can be customized inside
the [config.toml](../configuration.md) file.

The default dashboard only listens on localhost:8081 per default. If you want to make it accessible from the Internet, you will need to change the default configuration (though we recommend using a reverse proxy).
