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
    type Error = u8;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(WsTopic::SyncStatus),
            1 => Ok(WsTopic::NodeStatus),
            2 => Ok(WsTopic::MPSMetrics),
            3 => Ok(WsTopic::TipSelectionMetrics),
            4 => Ok(WsTopic::Milestone),
            5 => Ok(WsTopic::PeerMetrics),
            6 => Ok(WsTopic::ConfirmedMilestoneMetrics),
            7 => Ok(WsTopic::Vertex),
            8 => Ok(WsTopic::SolidInfo),
            9 => Ok(WsTopic::ConfirmedInfo),
            10 => Ok(WsTopic::MilestoneInfo),
            11 => Ok(WsTopic::TipInfo),
            12 => Ok(WsTopic::DatabaseSizeMetrics),
            13 => Ok(WsTopic::DatabaseCleanupEvent),
            14 => Ok(WsTopic::SpamMetrics),
            15 => Ok(WsTopic::AverageSpamMetrics),
            _ => Err(val),
        }
    }
}
