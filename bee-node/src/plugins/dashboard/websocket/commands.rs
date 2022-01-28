// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[repr(u8)]
#[derive(Debug)]
pub(crate) enum WsCommand {
    Register = 0,
    Unregister = 1,
}

impl TryFrom<u8> for WsCommand {
    type Error = u8;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(WsCommand::Register),
            1 => Ok(WsCommand::Unregister),
            _ => Err(val),
        }
    }
}
