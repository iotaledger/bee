# Configuration

Bee uses the Toml standard as config file. If you are unsure about some syntax have a look at the official specs [here](https://toml.io).
The default config file is `config.toml`. You can change the path or name of the config file by using the `-c` or `--config` flag. 
For Example: `bee -c config_example.toml`

## Table of content

[//]: # "Table of contents created with: http://ecotrust-canada.github.io/markdown-toc/"

- [1. Logger](#1-logger)
  * [outputs](#outputs)
- [2. Network](#2-network)
    + [peering](#peering)
- [3. Protocol](#3-protocol)
  * [coordinator](#coordinator)
    + [public_key_ranges](#public-key-ranges)
  * [workers](#workers)
- [4. Rest_api](#4-rest-api)
- [5. Snapshot](#5-snapshot)
- [6. Pruning](#6-pruning)
- [7. Storage](#7-storage)
  * [storage](#storage)
  * [env](#env)
- [8. Tangle](#8-tangle)
- [9. Mqtt](#9-mqtt)
- [10. Dashboard](#10-dashboard)
  * [auth](#auth)

---

| Name       | Description                                                                                | Type   |
| :--------- | :----------------------------------------------------------------------------------------- | :----- |
| identity   | hex representation of an Ed25519 keypair. Can be generated with the `bee p2pidentity` tool | string |
| alias      | alias for your node. Shows up in dashboard                                                 | string |
| bech32_hrp | network address identifier                                                                 | string |
| network_id | network identifier                                                                         | string |

## 1. Logger

| Name                | Description                          | Type           |
| :------------------ | :----------------------------------- | :------------- |
| color_enabled       | stdout it colored if enabled         | bool           |
| target_width        | width of the target section of a log | integer[usize] |
| level_width         | width of the level section of a log  | integer[usize] |
| [outputs](#outputs) | config for different log filters     | array          |

### outputs

| Name         | Description             | Type   |
| :----------- | :---------------------- | :----- |
| name         | standart stream or file | string |
| level_filter | log level filter        | string |

Example:

```toml
[logger]
color_enabled = true
target_width = 42
level_width = 5
[[logger.outputs]]
name          = "stdout"
level_filter  = "info" # other possible values are: "error", "warn", "info", "debug", "trace"
[[logger.outputs]]
name          = "error.log"
level_filter  = "error"
```

## 2. Network

| Name                    | Description                                                 | Type              |
| :---------------------- | :---------------------------------------------------------- | :---------------- |
| bind_address            | the address/es the networking layer tries binding to        | string[Multiaddr] |
| reconnect_interval_secs | the automatic reconnect interval in seconds for known peers | integer[u64]      |
| max_unknown_peers       | max count of allowed unknown peers                          | integer[usize]    |
| [peering](#peering)     | array of static peers                                       | array of tables   |

#### peering

| Name    | Description                                                                                          | Type   |
| :------ | :--------------------------------------------------------------------------------------------------- | :----- |
| address | libp2p formatted address(PeerID can be found on the dashboard or in the logs. It starts with `12D3`) | string |
| alias   | alias of the peer                                                                                    | string |

Example:

```toml
[network]
bind_address             = "/ip4/0.0.0.0/tcp/15600"
reconnect_interval_secs  = 30
max_unknown_peers        = 4

[network.peering]
[[network.peering.peers]]
address = "/ip4/192.0.2.0/tcp/15600/p2p/PeerID"
alias   = "some peer"
[[network.peering.peers]]
address = "/ip6/2001:db8::/tcp/15600/p2p/PeerID"
alias   = "another peer"
[[network.peering.peers]]
address = "/dns/example.com/tcp/15600/p2p/PeerID"
alias   = "yet another peer"
```

## 3. Protocol

| Name                        | Description           | Type         |
| :-------------------------- | :-------------------- | :----------- |
| minimum_pow_score           | the minimum pow score | float[f64]   |
| [coordinator](#coordinator) | coordinator configs   | table        |
| [workers](#workers)         | worker configs        | table        |

### coordinator

| Name                                    | Description           | Type            |
| :-------------------------------------- | :-------------------- | :-------------- |
| public_key_count                        | number of public keys | integer[usize]  |
| [public_key_ranges](#public_key_ranges) | public key ranges     | array of tables |

#### public_key_ranges

| Name       | Description | Type         |
| :--------- | :---------- | :----------- |
| public_key | public key  | string       |
| start      | start       | integer[u32] |
| end        | end         | integer[u32] |

### workers

| Name                 | Description           | Type           |
| :------------------- | :-------------------- | :------------- |
| message_worker_cache | TO-DO                 | integer[usize] |
| status_interval      | status interval in ms | integer[u64]   |
| ms_sync_count        | milestone sync count  | integer[u32]   |

Example:

```toml
[protocol]
minimum_pow_score = 4000
handshake_window = 10
[protocol.coordinator]
public_key_count  = 2
[[protocol.coordinator.public_key_ranges]]
public_key  = "7205c145525cee64f1c9363696811d239919d830ad964b4e29359e6475848f5a"
start       = 0
end         = 0
[[protocol.coordinator.public_key_ranges]]
public_key  = "e468e82df33d10dea3bd0eadcd7867946a674d207c39f5af4cc44365d268a7e6"
start       = 0
end         = 0
[[protocol.coordinator.public_key_ranges]]
public_key  = "0758028d34508079ba1f223907ac3bb5ce8f6bdccc6b961c7c85a2f460b30c1d"
start       = 0
end         = 0
[protocol.workers]
message_worker_cache = 10000
status_interval = 10
ms_sync_count = 200
```

## 4. Rest_api

| Name                              | Description                       | Type             |
| :-------------------------------- | :-------------------------------- | :--------------- |
| binding_port                      | binding port for rest API         | integer[u16]     |
| binding_ip_addr                   | binding address for rest API      | string[IpAddr]   |
| feature_proof_of_work             | enable pow                        | bool             |
| white_flag_solidification_timeout | white flag solidification timeout | integer[u64]     |
| public_routes                     | API routes which should be public | array of strings |
| allowed_ips                       | list of whitelisted IPs           | string[IpAddr]   |

Example:

```toml
[rest_api]
binding_port          = 14265
binding_ip_addr       = "0.0.0.0"
feature_proof_of_work = true
public_routes         = [
    "/api/v1/peers",
    "/api/v1/addresses/:address",
    "/api/v1/addresses/ed25519/:address",
    "/health",
    "/api/v1/info",
    "/api/v1/messages/:messageId",
    "/api/v1/messages/:messageId/children",
    "/api/v1/messages/:messageId/metadata",
    "/api/v1/messages/:messageId/raw",
    "/api/v1/messages",
    "/api/v1/milestones/:milestoneIndex",
    "/api/v1/milestones/:milestoneIndex/utxo-changes",
    "/api/v1/outputs/:outputId",
    "/api/v1/addresses/:address/outputs",
    "/api/v1/addresses/ed25519/:address/outputs",
    "/api/v1/peers/:peerId",
    "/api/v1/peers",
    "/api/v1/peers/:peerId",
    "/api/v1/messages",
    "/api/v1/messages",
    "/api/v1/tips",
    "/api/v1/receipts",
    "/api/v1/receipts/:milestoneIndex",
    "/api/v1/treasury",
    "/api/v1/transactions/:transactionId/included-message",
    "/api/plugins/debug/whiteflag",
]
allowed_ips = [
    "127.0.0.1",
    "::1"
]
white_flag_solidification_timeout = 2
```

## 5. Snapshot

| Name              | Description                                                                                | Type             |
| :---------------- | :----------------------------------------------------------------------------------------- | :--------------- |
| full_path         | path to the full snapshot file                                                             | string           |
| delta_path        | path to the delta snapshot file                                                            | string           |
| download_urls     | list of download URLs for the snapshot                                                     | array of strings |
| depth             | the depth, respectively the starting point, at which a snapshot of the ledger is generated | integer[u32]     |
| interval_synced   | interval, in milestones, at which snapshot files are created if the node is sync           | integer[u32]     |
| interval_unsynced | interval, in milestones, at which snapshot files are created if the node is unsync         | integer[u32]     |

Example:

```toml
[snapshot]
full_path         = "./snapshots/alphanet/full_snapshot.bin"
delta_path        = "./snapshots/alphanet/delta_snapshot.bin"
download_urls     = [
  "https://dbfiles.testnet.chrysalis2.com/",
]
depth             = 50
interval_synced   = 50
interval_unsynced = 1000
```

## 6. Pruning

| Name           | Description                                           | Type         |
| :------------- | :---------------------------------------------------- | :----------- |
| enabled        | enable pruning                                        | bool         |
| delay          | amount of milestone cones to keep in the database     | integer[u32] |
| prune_receipts | whether to delete old receipts data from the database | bool         |

Example:

```toml
[pruning]
enabled         = true
delay           = 60480
prune_receipts  = false
```

## 7. Storage

| Name                                       | Description          | Type           |
| :----------------------------------------- | :------------------- | :------------- |
| path                                       | path to the database | string         |
| create_if_missing                          | TO-DO                | bool           |
| create_missing_column_families             | TO-DO                | bool           |
| enable_statistics                          | TO-DO                | bool           |
| increase_parallelism                       | TO-DO                | integer[i32]   |
| optimize_for_point_lookup                  | TO-DO                | integer[u64]   |
| optimize_level_style_compaction            | TO-DO                | integer[usize] |
| optimize_universal_style_compaction        | TO-DO                | integer[usize] |
| set_advise_random_on_open                  | TO-DO                | bool           |
| set_allow_concurrent_memtable_write        | TO-DO                | bool           |
| set_allow_mmap_reads                       | TO-DO                | bool           |
| set_allow_mmap_writes                      | TO-DO                | bool           |
| set_atomic_flush                           | TO-DO                | bool           |
| set_bytes_per_sync                         | TO-DO                | integer[u64]   |
| set_compaction_readahead_size              | TO-DO                | integer[usize] |
| set_max_write_buffer_number                | TO-DO                | integer[i32]   |
| set_write_buffer_size                      | TO-DO                | integer[usize] |
| set_db_write_buffer_size                   | TO-DO                | integer[usize] |
| set_disable_auto_compactions               | TO-DO                | bool           |
| set_unordered_write                        | TO-DO                | bool           |
| set_use_direct_io_for_flush_and_compaction | TO-DO                | bool           |
| [storage](#storage)                        | TO-DO                | table          |
| set_compaction_style                       | TO-DO                | string         |
| set_compression_type                       | TO-DO                | string         |
| [env](#env)                                | TO-DO                | table          |

### storage

| Name                  | Description | Type           |
| :-------------------- | :---------- | :------------- |
| fetch_edge_limit      | TO-DO       | integer[usize] |
| fetch_index_limit     | TO-DO       | integer[usize] |
| fetch_output_id_limit | TO-DO       | integer[usize] |
| iteration_budget      | TO-DO       | integer[usize] |

### env

| Name                                 | Description | Type         |
| :----------------------------------- | :---------- | :----------- |
| set_background_threads               | TO-DO       | integer[i32] |
| set_high_priority_background_threads | TO-DO       | integer[i32] |

Example:

```toml
[storage]
path = "./storage/alphanet"
create_if_missing = true
create_missing_column_families = true
enable_statistics = false
increase_parallelism = 4 # defaults to the number of cpu cores
optimize_for_point_lookup = 67108864 # 64 MiB
optimize_level_style_compaction = 0
optimize_universal_style_compaction = 0
set_advise_random_on_open = true
set_allow_concurrent_memtable_write = true
set_allow_mmap_reads = false
set_allow_mmap_writes = false
set_atomic_flush = false
set_bytes_per_sync = 0
set_compaction_readahead_size = 0
set_max_write_buffer_number = 2
set_write_buffer_size = 67108864 # 64 MiB
set_db_write_buffer_size = 67108864 # 64 MiB
set_disable_auto_compactions = false
set_unordered_write = true
set_use_direct_io_for_flush_and_compaction = true
set_compaction_style = "Fifo" # other possible values are: "Level", "Universal"
set_compression_type = "None" # other possible values are: "Snappy", "Zlib", "Bz2", "Lz4", "Lz4hc", "Zstd"
[storage.storage]
fetch_edge_limit = 1000
fetch_index_limit = 1000
fetch_output_id_limit = 1000
iteration_budget = 100
[storage.env]
set_background_threads = 4 # defaults to the number of cpu cores
set_high_priority_background_threads = 2
```

## 8. Tangle

| Name            | Description | Type   |
| :-------------- | :---------- | :----- |
| below_max_depth | TO-DO     | string |

Example
```toml
[tangle]
below_max_depth = 15
```

## 9. Mqtt

| Name    | Description | Type   |
| :------ | :---------- | :----- |
| address | address     | string |

Example:

```toml
[mqtt]
address = "tcp://localhost:1883"
```

## 10. Dashboard

| Name | Description    | Type         |
| :--- | :------------- | :----------- |
| port | dashboard port | integer[u16] |
| auth | dashboard auth | table        |

### auth

| Name            | Description                                                  | Type         |
| :-------------- | :----------------------------------------------------------- | :----------- |
| session_timeout | expiration time of the authentication in seconds             | integer[u64] |
| user            | user                                                         | String       |
| password_salt   | password salt. Can be generated with the `bee password` tool | String       |
| password_hash   | password hash. Can be generated with the `bee password` tool | String       |

Example:

```toml
[dashboard]
port  = 8081
[dashboard.auth]
session_timeout = 86400
user            = "admin"
password_salt   = "0000000000000000000000000000000000000000000000000000000000000000"
password_hash   = "0000000000000000000000000000000000000000000000000000000000000000"
```
