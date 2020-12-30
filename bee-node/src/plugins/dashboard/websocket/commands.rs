// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::convert::TryFrom;

#[repr(u8)]
#[derive(Debug)]
pub(crate) enum WsCommand {
    Register = 0,
    Unregister = 1,
}

impl TryFrom<u8> for WsCommand {
    type Error = String;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x0 => Ok(WsCommand::Register),
            0x1 => Ok(WsCommand::Unregister),
            _ => Err("unknown command".to_string()),
        }
    }
}
