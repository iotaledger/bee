// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

#[derive(Copy, Clone)]
pub struct Offset {
    pub start: usize,
    pub length: usize,
}

#[derive(Copy, Clone)]
pub struct Field {
    pub trit_offset: Offset,
    pub tryte_offset: Offset,
}

impl Field {
    pub fn byte_start(&self) -> usize {
        self.trit_offset.start / 5
    }

    pub fn byte_length(&self) -> usize {
        if self.trit_offset.length % 5 == 0 {
            self.trit_offset.length / 5
        } else {
            self.trit_offset.length / 5 + 1
        }
    }
}

macro_rules! offsets_from_trits {
    ($start:expr, $length:expr) => {
        Field {
            trit_offset: Offset {
                start: $start,
                length: $length,
            },
            tryte_offset: Offset {
                start: $start / 3,
                length: $length / 3,
            },
        }
    };
}

macro_rules! offsets_from_previous_field {
    ($prev:expr, $length:expr) => {
        Field {
            trit_offset: Offset {
                start: ($prev).trit_offset.start + ($prev).trit_offset.length,
                length: $length,
            },
            tryte_offset: Offset {
                start: (($prev).trit_offset.start + ($prev).trit_offset.length) / 3,
                length: $length / 3,
            },
        }
    };
}

pub const IOTA_SUPPLY: i64 = 2_779_530_283_277_761;

pub const TRANSACTION_TRIT_LEN: usize = 8019;
pub const TRANSACTION_TRYT_LEN: usize = TRANSACTION_TRIT_LEN / 3; // 2673
pub const TRANSACTION_BYTE_LEN: usize = TRANSACTION_TRIT_LEN / 5 + 1; // 1604

pub const PAYLOAD_TRIT_LEN: usize = 6561;
pub const ADDRESS_TRIT_LEN: usize = 243;
pub const VALUE_TRIT_LEN: usize = 81;
pub const TAG_TRIT_LEN: usize = 81;
pub const TIMESTAMP_TRIT_LEN: usize = 27;
pub const INDEX_TRIT_LEN: usize = 27;
pub const HASH_TRIT_LEN: usize = 243;
pub const NONCE_TRIT_LEN: usize = 81;

pub(crate) const PAYLOAD: Field = offsets_from_trits!(0, PAYLOAD_TRIT_LEN);
pub(crate) const ADDRESS: Field = offsets_from_previous_field!(PAYLOAD, ADDRESS_TRIT_LEN);
pub(crate) const VALUE: Field = offsets_from_previous_field!(ADDRESS, VALUE_TRIT_LEN);
pub(crate) const OBSOLETE_TAG: Field = offsets_from_previous_field!(VALUE, TAG_TRIT_LEN);
pub(crate) const TIMESTAMP: Field = offsets_from_previous_field!(OBSOLETE_TAG, TIMESTAMP_TRIT_LEN);
pub(crate) const INDEX: Field = offsets_from_previous_field!(TIMESTAMP, INDEX_TRIT_LEN);
pub(crate) const LAST_INDEX: Field = offsets_from_previous_field!(INDEX, INDEX_TRIT_LEN);
pub(crate) const BUNDLE: Field = offsets_from_previous_field!(LAST_INDEX, HASH_TRIT_LEN);
pub(crate) const TRUNK: Field = offsets_from_previous_field!(BUNDLE, HASH_TRIT_LEN);
pub(crate) const BRANCH: Field = offsets_from_previous_field!(TRUNK, HASH_TRIT_LEN);
pub(crate) const TAG: Field = offsets_from_previous_field!(BRANCH, TAG_TRIT_LEN);
pub(crate) const ATTACHMENT_TS: Field = offsets_from_previous_field!(TAG, TIMESTAMP_TRIT_LEN);
pub(crate) const ATTACHMENT_LBTS: Field = offsets_from_previous_field!(ATTACHMENT_TS, TIMESTAMP_TRIT_LEN);
pub(crate) const ATTACHMENT_UBTS: Field = offsets_from_previous_field!(ATTACHMENT_LBTS, TIMESTAMP_TRIT_LEN);
pub(crate) const NONCE: Field = offsets_from_previous_field!(ATTACHMENT_UBTS, NONCE_TRIT_LEN);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn add_up_to_transaction_trit_length() {
        let total_trit_length = PAYLOAD.trit_offset.length
            + ADDRESS.trit_offset.length
            + VALUE.trit_offset.length
            + OBSOLETE_TAG.trit_offset.length
            + TIMESTAMP.trit_offset.length
            + INDEX.trit_offset.length
            + LAST_INDEX.trit_offset.length
            + BUNDLE.trit_offset.length
            + TRUNK.trit_offset.length
            + BRANCH.trit_offset.length
            + TAG.trit_offset.length
            + ATTACHMENT_TS.trit_offset.length
            + ATTACHMENT_LBTS.trit_offset.length
            + ATTACHMENT_UBTS.trit_offset.length
            + NONCE.trit_offset.length;

        assert_eq!(total_trit_length, TRANSACTION_TRIT_LEN);
    }

    #[test]
    fn add_up_to_transaction_tryte_length() {
        let total_tryte_length = PAYLOAD.tryte_offset.length
            + ADDRESS.tryte_offset.length
            + VALUE.tryte_offset.length
            + OBSOLETE_TAG.tryte_offset.length
            + TIMESTAMP.tryte_offset.length
            + INDEX.tryte_offset.length
            + LAST_INDEX.tryte_offset.length
            + BUNDLE.tryte_offset.length
            + TRUNK.tryte_offset.length
            + BRANCH.tryte_offset.length
            + TAG.tryte_offset.length
            + ATTACHMENT_TS.tryte_offset.length
            + ATTACHMENT_LBTS.tryte_offset.length
            + ATTACHMENT_UBTS.tryte_offset.length
            + NONCE.tryte_offset.length;

        assert_eq!(total_tryte_length, TRANSACTION_TRYT_LEN);
    }
}
