// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::error::Error;

use bee_message::milestone::MilestoneIndex;

use log::info;

#[allow(dead_code)]
pub(crate) fn snapshot(_full_path: &std::path::Path, _snapshot_index: MilestoneIndex) -> Result<(), Error> {
    info!("Snapshotting...");
    // TODO
    Ok(())
}
