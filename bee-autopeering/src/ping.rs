// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::proto;

pub(crate) struct Ping(proto::Ping);

// impl Ping {
//     pub fn new() -> Self {
//         Self(proto::Ping {
//             dst_addr,
//             network_id,
//             src_addr,
//             src_port,
//             timestamp,
//             version,
//         })
//     }
// }
