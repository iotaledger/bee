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

use crate::{
    bundled::{
        constants::{
            Field, ADDRESS, ATTACHMENT_LBTS, ATTACHMENT_TS, ATTACHMENT_UBTS, BRANCH, BUNDLE, INDEX, LAST_INDEX, NONCE,
            OBSOLETE_TAG, PAYLOAD, TAG, TIMESTAMP, TRANSACTION_TRIT_LEN, TRUNK, VALUE,
        },
        Address, BundledTransactionBuilder, BundledTransactionField, Index, Nonce, Payload, Tag, Timestamp, Value,
    },
    Vertex,
};

use bee_crypto::ternary::Hash;
use bee_ternary::{convert::Error as ConvertError, raw::RawEncoding, Btrit, T1B1Buf, TritBuf, Trits, T1B1};

use std::convert::TryFrom;

#[derive(Debug)]
pub enum BundledTransactionError {
    InvalidNumericField(&'static str, ConvertError),
    MissingField(&'static str),
    InvalidValue(i64),
    InvalidAddress,
}

#[derive(PartialEq, Clone, Debug)]
pub struct BundledTransaction {
    pub(crate) payload: Payload,
    pub(crate) address: Address,
    pub(crate) value: Value,
    pub(crate) obsolete_tag: Tag,
    pub(crate) timestamp: Timestamp,
    pub(crate) index: Index,
    pub(crate) last_index: Index,
    pub(crate) bundle: Hash,
    pub(crate) trunk: Hash,
    pub(crate) branch: Hash,
    pub(crate) tag: Tag,
    pub(crate) attachment_ts: Timestamp,
    pub(crate) attachment_lbts: Timestamp,
    pub(crate) attachment_ubts: Timestamp,
    pub(crate) nonce: Nonce,
}

impl Eq for BundledTransaction {}

impl BundledTransaction {
    pub fn from_trits(
        buffer: &Trits<impl RawEncoding<Trit = Btrit> + ?Sized>,
    ) -> Result<Self, BundledTransactionError> {
        let trits = buffer.encode::<T1B1Buf>();

        let transaction = BundledTransactionBuilder::new()
            .with_payload(Payload(
                trits[PAYLOAD.trit_offset.start..PAYLOAD.trit_offset.start + PAYLOAD.trit_offset.length].to_buf(),
            ))
            .with_address(Address(
                trits[ADDRESS.trit_offset.start..ADDRESS.trit_offset.start + ADDRESS.trit_offset.length].to_buf(),
            ))
            .with_value(Value::from_inner_unchecked(
                i64::try_from(&trits[VALUE.trit_offset.start..VALUE.trit_offset.start + VALUE.trit_offset.length])
                    .map_err(|e| BundledTransactionError::InvalidNumericField("value", e))?,
            ))
            .with_obsolete_tag(Tag(trits[OBSOLETE_TAG.trit_offset.start
                ..OBSOLETE_TAG.trit_offset.start + OBSOLETE_TAG.trit_offset.length]
                .to_buf()))
            .with_timestamp(Timestamp::from_inner_unchecked(
                i128::try_from(
                    &trits[TIMESTAMP.trit_offset.start..TIMESTAMP.trit_offset.start + TIMESTAMP.trit_offset.length],
                )
                .map_err(|e| BundledTransactionError::InvalidNumericField("timestamp", e))? as u64,
            ))
            .with_index(Index::from_inner_unchecked(
                i128::try_from(&trits[INDEX.trit_offset.start..INDEX.trit_offset.start + INDEX.trit_offset.length])
                    .map_err(|e| BundledTransactionError::InvalidNumericField("index", e))? as usize,
            ))
            .with_last_index(Index::from_inner_unchecked(
                i128::try_from(
                    &trits[LAST_INDEX.trit_offset.start..LAST_INDEX.trit_offset.start + LAST_INDEX.trit_offset.length],
                )
                .map_err(|e| BundledTransactionError::InvalidNumericField("last_index", e))? as usize,
            ))
            .with_tag(Tag(trits
                [TAG.trit_offset.start..TAG.trit_offset.start + TAG.trit_offset.length]
                .to_buf()))
            .with_attachment_ts(Timestamp::from_inner_unchecked(
                i128::try_from(
                    &trits[ATTACHMENT_TS.trit_offset.start
                        ..ATTACHMENT_TS.trit_offset.start + ATTACHMENT_TS.trit_offset.length],
                )
                .map_err(|e| BundledTransactionError::InvalidNumericField("attachment_ts", e))? as u64,
            ))
            .with_bundle(Hash::from_inner_unchecked(
                trits[BUNDLE.trit_offset.start..BUNDLE.trit_offset.start + BUNDLE.trit_offset.length].to_buf(),
            ))
            .with_trunk(Hash::from_inner_unchecked(
                trits[TRUNK.trit_offset.start..TRUNK.trit_offset.start + TRUNK.trit_offset.length].to_buf(),
            ))
            .with_branch(Hash::from_inner_unchecked(
                trits[BRANCH.trit_offset.start..BRANCH.trit_offset.start + BRANCH.trit_offset.length].to_buf(),
            ))
            .with_attachment_lbts(Timestamp::from_inner_unchecked(
                i128::try_from(
                    &trits[ATTACHMENT_LBTS.trit_offset.start
                        ..ATTACHMENT_LBTS.trit_offset.start + ATTACHMENT_LBTS.trit_offset.length],
                )
                .map_err(|e| BundledTransactionError::InvalidNumericField("attachment_lbts", e))?
                    as u64,
            ))
            .with_attachment_ubts(Timestamp::from_inner_unchecked(
                i128::try_from(
                    &trits[ATTACHMENT_UBTS.trit_offset.start
                        ..ATTACHMENT_UBTS.trit_offset.start + ATTACHMENT_UBTS.trit_offset.length],
                )
                .map_err(|e| BundledTransactionError::InvalidNumericField("attachment_ubts", e))?
                    as u64,
            ))
            .with_nonce(Nonce(
                trits[NONCE.trit_offset.start..NONCE.trit_offset.start + NONCE.trit_offset.length].to_buf(),
            ))
            .build()?;

        Ok(transaction)
    }

    pub fn into_trits_allocated(&self, buf: &mut Trits<T1B1>) {
        let mut copy_field = |layout: Field, field: &Trits<T1B1>| {
            buf[layout.trit_offset.start..][..layout.trit_offset.length].copy_from(&field[0..layout.trit_offset.length])
        };

        copy_field(PAYLOAD, self.payload().to_inner());
        copy_field(ADDRESS, self.address().to_inner());
        copy_field(OBSOLETE_TAG, self.obsolete_tag().to_inner());
        copy_field(BUNDLE, self.bundle().to_inner());
        copy_field(BRANCH, self.branch().to_inner());
        copy_field(TRUNK, self.trunk().to_inner());
        copy_field(TAG, self.tag().to_inner());
        copy_field(NONCE, self.nonce().to_inner());

        let mut copy_slice =
            |layout: Field, slice: &Trits<T1B1>| buf[layout.trit_offset.start..][..slice.len()].copy_from(slice);

        let value_buf = TritBuf::<T1B1Buf<_>>::from(*self.value().to_inner());
        copy_slice(VALUE, &value_buf);

        let index_buf = TritBuf::<T1B1Buf<_>>::from(*self.index().to_inner() as i128);
        copy_slice(INDEX, &index_buf);

        let last_index_buf = TritBuf::<T1B1Buf<_>>::from(*self.last_index().to_inner() as i128);
        copy_slice(LAST_INDEX, &last_index_buf);

        let timestamp_buf = TritBuf::<T1B1Buf<_>>::from(*self.timestamp().to_inner() as i128);
        copy_slice(TIMESTAMP, &timestamp_buf);

        let attachment_ts_buf = TritBuf::<T1B1Buf<_>>::from(*self.attachment_ts().to_inner() as i128);
        copy_slice(ATTACHMENT_TS, &attachment_ts_buf);

        let attachment_lbts_buf = TritBuf::<T1B1Buf<_>>::from(*self.attachment_lbts().to_inner() as i128);
        copy_slice(ATTACHMENT_LBTS, &attachment_lbts_buf);

        let attachment_ubts_buf = TritBuf::<T1B1Buf<_>>::from(*self.attachment_ubts().to_inner() as i128);
        copy_slice(ATTACHMENT_UBTS, &attachment_ubts_buf);
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn obsolete_tag(&self) -> &Tag {
        &self.obsolete_tag
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    pub fn last_index(&self) -> &Index {
        &self.last_index
    }

    pub fn bundle(&self) -> &Hash {
        &self.bundle
    }

    pub fn tag(&self) -> &Tag {
        &self.tag
    }

    pub fn attachment_ts(&self) -> &Timestamp {
        &self.attachment_ts
    }

    pub fn attachment_lbts(&self) -> &Timestamp {
        &self.attachment_lbts
    }

    pub fn attachment_ubts(&self) -> &Timestamp {
        &self.attachment_ubts
    }

    pub fn nonce(&self) -> &Nonce {
        &self.nonce
    }

    pub fn is_tail(&self) -> bool {
        self.index == Index(0)
    }

    pub fn is_head(&self) -> bool {
        self.index == self.last_index
    }

    // TODO rename ?
    // TODO return type ?
    pub fn get_timestamp(&self) -> u64 {
        match self.attachment_ts.to_inner() {
            0 => *self.timestamp.to_inner(),
            _ => *self.attachment_ts.to_inner() / 1000,
        }
    }

    pub const fn trit_len() -> usize {
        TRANSACTION_TRIT_LEN
    }
}

impl Vertex for BundledTransaction {
    type Hash = Hash;

    fn trunk(&self) -> &Self::Hash {
        &self.trunk
    }

    fn branch(&self) -> &Self::Hash {
        &self.branch
    }
}

#[derive(Default)]
pub struct BundledTransactions(pub(crate) Vec<BundledTransaction>);

impl BundledTransactions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, index: usize) -> Option<&BundledTransaction> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, transaction: BundledTransaction) {
        self.0.push(transaction);
    }
}
