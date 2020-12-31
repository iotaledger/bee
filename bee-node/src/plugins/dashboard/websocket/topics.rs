// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde_repr::Serialize_repr;

use std::convert::TryFrom;

#[repr(u8)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize_repr)]
pub(crate) enum WsTopic {
    SyncStatus = 0,
    NodeStatus = 1,
    MPSMetrics = 2,
    TipSelectionMetrics = 3,
    Milestone = 4,
    PeerMetrics = 5,
    ConfirmedMilestoneMetrics = 6,
    Vertex = 7,
    SolidInfo = 8,
    ConfirmedInfo = 9,
    MilestoneInfo = 10,
    TipInfo = 11,
    DatabaseSizeMetrics = 12,
    DatabaseCleanupEvent = 13,
    SpamMetrics = 14,
    AverageSpamMetrics = 15,
}

impl TryFrom<u8> for WsTopic {
    type Error = String;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x0 => Ok(WsTopic::SyncStatus),
            0x1 => Ok(WsTopic::NodeStatus),
            0x2 => Ok(WsTopic::MPSMetrics),
            0x3 => Ok(WsTopic::TipSelectionMetrics),
            0x4 => Ok(WsTopic::Milestone),
            0x5 => Ok(WsTopic::PeerMetrics),
            0x6 => Ok(WsTopic::ConfirmedMilestoneMetrics),
            0x7 => Ok(WsTopic::Vertex),
            0x8 => Ok(WsTopic::SolidInfo),
            0x9 => Ok(WsTopic::ConfirmedInfo),
            0x10 => Ok(WsTopic::MilestoneInfo),
            0x11 => Ok(WsTopic::TipInfo),
            0x12 => Ok(WsTopic::DatabaseSizeMetrics),
            0x13 => Ok(WsTopic::DatabaseCleanupEvent),
            0x14 => Ok(WsTopic::SpamMetrics),
            0x15 => Ok(WsTopic::AverageSpamMetrics),
            _ => Err(format!("Unknown websocket topic: {}.", val)),
        }
    }
}
