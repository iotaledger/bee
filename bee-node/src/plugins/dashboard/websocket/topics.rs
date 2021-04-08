// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde_repr::Serialize_repr;

use std::convert::TryFrom;

#[repr(u8)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize_repr)]
pub(crate) enum WsTopic {
    SyncStatus = 0,
    PublicNodeStatus = 1,
    NodeStatus = 2,
    MpsMetrics = 3,
    TipSelectionMetrics = 4,
    Milestone = 5,
    PeerMetrics = 6,
    ConfirmedMilestoneMetrics = 7,
    Vertex = 8,
    SolidInfo = 9,
    ConfirmedInfo = 10,
    MilestoneInfo = 11,
    TipInfo = 12,
    DatabaseSizeMetrics = 13,
    DatabaseCleanupEvent = 14,
    SpamMetrics = 15,
    AverageSpamMetrics = 16,
}

impl TryFrom<u8> for WsTopic {
    type Error = u8;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(WsTopic::SyncStatus),
            1 => Ok(WsTopic::PublicNodeStatus),
            2 => Ok(WsTopic::NodeStatus),
            3 => Ok(WsTopic::MpsMetrics),
            4 => Ok(WsTopic::TipSelectionMetrics),
            5 => Ok(WsTopic::Milestone),
            6 => Ok(WsTopic::PeerMetrics),
            7 => Ok(WsTopic::ConfirmedMilestoneMetrics),
            8 => Ok(WsTopic::Vertex),
            9 => Ok(WsTopic::SolidInfo),
            10 => Ok(WsTopic::ConfirmedInfo),
            11 => Ok(WsTopic::MilestoneInfo),
            12 => Ok(WsTopic::TipInfo),
            13 => Ok(WsTopic::DatabaseSizeMetrics),
            14 => Ok(WsTopic::DatabaseCleanupEvent),
            15 => Ok(WsTopic::SpamMetrics),
            16 => Ok(WsTopic::AverageSpamMetrics),
            _ => Err(val),
        }
    }
}

impl WsTopic {
    pub fn is_public(&self) -> bool {
        matches!(
            self,
            WsTopic::SyncStatus
                | WsTopic::PublicNodeStatus
                | WsTopic::MpsMetrics
                | WsTopic::Milestone
                | WsTopic::ConfirmedMilestoneMetrics
                | WsTopic::Vertex
                | WsTopic::SolidInfo
                | WsTopic::ConfirmedInfo
                | WsTopic::MilestoneInfo
                | WsTopic::TipInfo
        )
    }
}
