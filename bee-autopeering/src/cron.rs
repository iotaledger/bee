// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

// Command run on notice
#[async_trait::async_trait]
pub(crate) trait CronJob
where
    Self: Send,
{
    type Command: Send;
    type Data: Send;

    async fn cronjob(self, period: Duration, cmd: Self::Command, data: Self::Data);
}
