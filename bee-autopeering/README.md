# bee-autopeering

A system that allows peers in the same IOTA network to automatically discover each other.

In order to integrate the Autopeering functionality in your node implementation you need to provide its `init` function with the following data:
* an `AutopeeringConfig`;
* the protocol version (`u32`);
* a network name, e.g. "chrysalis-mainnet";
* a `Local` entity (either randomly created or from an `Ed25519` keypair), that additionally announces one or more services;
* a shutdown signal (`Future`);
* a peer store, e.g. the `InMemoryPeerStore` (non-persistent) or the `SledPeerStore` (persistent), or a custom peer store implementing the `PeerStore` trait;

## Example

```rust

use bee_autopeering::{
    init,
    peerstore::{SledPeerStore, SledPeerStoreConfig},
    AutopeeringConfig, Event, Local, NeighborValidator, Peer, ServiceProtocol, AUTOPEERING_SERVICE_NAME,
};

const NETWORK: &str = "chrysalis-mainnet";

// An example autopeering config in JSON format:
fn read_config() -> AutopeeringConfig {
    let config_json = r#"
    {
        "bindAddress": "0.0.0.0:14627",
        "entryNodes": [
            "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb",
            "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2",
        ],
        "entryNodesPreferIPv6": false,
        "runAsEntryNode": false
    }"#;

    serde_json::from_str(config_json).expect("error deserializing json config")
}

#[tokio::main]
async fn main() {
    // Peers will only accept each other as peer if they agree on the protocol version and the
    // network name.
    const VERSION: u32 = 1;

    // Read the config from a JSON file/string.
    let config = read_config();

    // Create a random local entity, that announces two services:
    let local = {
        let l = Local::default();
        let mut write = l.write();
        write.add_service(AUTOPEERING_SERVICE_NAME, ServiceProtocol::Udp, config.bind_addr.port());
        write.add_service(NETWORK, ServiceProtocol::Tcp, 15600);
        drop(write)
        l
    };

    // You can choose between the `InMemoryPeerStore` (non-persistent) and the `SledPeerStore` 
    // (persistent).
    let peerstore_config = SledPeerStoreConfig::new().path("./peerstore");

    // The `NeighborValidator` allows you to customize the peer selection.
    let neighbor_validator = GossipNeighborValidator {};

    // We need to provide a shutdown signal (can basically be any `Future`).
    let term_signal = tokio::signal::ctrl_c();

    // Initialize the autopeering functionality.
    let mut event_rx = bee_autopeering::init::<SledPeerStore, _, _, GossipNeighborValidator>(
        config.clone(),
        VERSION,
        NETWORK,
        local,
        peerstore_config,
        term_signal,
        neighbor_validator,
    )
    .await
    .expect("initializing autopeering system failed");

    // Process autopeering events.
    loop {
        tokio::select! {
            e = event_rx.recv() => {
                if let Some(event) = e {
                    // handle the event
                    // process(event);
                } else {
                    break;
                }
            }
        };
    }
}

#[derive(Clone)]
struct GossipNeighborValidator {}

impl NeighborValidator for GossipNeighborValidator {
    fn is_valid(&self, peer: &Peer) -> bool {
        peer.has_service(NETWORK)
    }
}

```
