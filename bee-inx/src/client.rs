// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use ::inx::{proto::inx_client::InxClient, tonic};

use crate::error::Error;

/// An INX client connection.
#[derive(Clone, Debug)]
pub struct Inx {
    pub(crate) client: InxClient<tonic::transport::Channel>,
}

impl Inx {
    /// Connect to the INX interface of a node.
    pub async fn connect(address: impl ToString) -> Result<Self, Error> {
        Ok(Self {
            client: InxClient::connect(address.to_string()).await?,
        })
    }
}

pub(crate) fn try_from_inx_type<P, B>(msg: Result<P, tonic::Status>) -> Result<B, Error>
where
    B: TryFrom<P, Error = bee_block::InxError>,
{
    let inner = msg.map_err(Error::StatusCode)?;
    B::try_from(inner).map_err(Error::InxError)
}
