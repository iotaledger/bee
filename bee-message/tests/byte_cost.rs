// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::identity_op)]

use bee_byte_cost::{ByteCost, ByteCostConfig};
use bee_common::packable::Packable;

use std::fmt::Debug;

type Weight = u64;

const KEY: Weight = 10;
const DATA: Weight = 1;
const KEY_DATA: Weight = KEY + DATA;

const CONFIG: ByteCostConfig = ByteCostConfig {
    byte_cost: 1,
    weight_for_data: DATA,
    weight_for_key: KEY,
    both_indexation_and_sender: false,
};

fn test_primitive<P>(primitive: P, target: &[Weight])
where
    P: ByteCost + Debug + Packable,
{
    assert_eq!(
        primitive.weighted_bytes(&CONFIG),
        target.iter().sum::<u64>(),
        "{:#?}",
        primitive
    );
}

fn test_primitives<'a, P>(primitives: &'a [(P, &'a [Weight])])
where
    P: ByteCost + Debug + Packable + Clone + 'a,
{
    for (primitive, target) in primitives {
        test_primitive(primitive.clone(), target)
    }
}

#[cfg(test)]
mod feature_block {
    use super::*;

    use bee_message::output::feature_block::*;
    use bee_test::rand::{address::*, bytes::rand_bytes, milestone::*};

    #[test]
    fn indexation_feature_block() {
        test_primitive(
            IndexationFeatureBlock::new(&rand_bytes(IndexationFeatureBlock::LENGTH_MAX)).unwrap(),
            &[
                1 * DATA,      // Indexation Data Length,
                64 * KEY_DATA, // Indexation Data
            ],
        );
    }

    #[test]
    fn metadata_feature_block() {
        test_primitive(
            MetadataFeatureBlock::new(&[1, 2, 3, 4]).unwrap(),
            &[
                4 * DATA, // Metadata Data Length
                4 * DATA, // Metadata Data
            ],
        );
    }

    #[test]
    fn sender_feature_block() {
        test_primitives(&[
            (
                SenderFeatureBlock::new(rand_alias_address().into()),
                &[21 * KEY_DATA /* Sender */],
            ),
            (
                SenderFeatureBlock::new(rand_ed25519_address().into()),
                &[33 * KEY_DATA /* Sender */],
            ),
            (
                SenderFeatureBlock::new(rand_nft_address().into()),
                &[21 * KEY_DATA /* Sender */],
            ),
        ]);
    }

    #[test]
    fn timelock_milestone_index_feature_block() {
        test_primitive(
            TimelockMilestoneIndexFeatureBlock::new(rand_milestone_index()),
            &[4 * DATA /* Milestone Index */],
        );
    }

    #[test]
    fn timelock_unix_block() {
        test_primitive(TimelockUnixFeatureBlock::new(42), &[4 * DATA /* Unix Time */]);
    }

    #[test]
    fn expiration_milestone_index_block() {
        test_primitive(
            ExpirationMilestoneIndexFeatureBlock::new(rand_milestone_index()),
            &[4 * DATA /* Milestone Index */],
        );
    }

    #[test]
    fn expiration_unix_block() {
        test_primitive(ExpirationUnixFeatureBlock::new(42), &[4 * DATA /* Unix Time */]);
    }

    #[test]
    fn issuer_feature_block() {
        test_primitives(&[
            (
                IssuerFeatureBlock::new(rand_alias_address().into()),
                &[21 * KEY_DATA /* Issuer */],
            ),
            (
                IssuerFeatureBlock::new(rand_ed25519_address().into()),
                &[33 * KEY_DATA /* Issuer */],
            ),
            (
                IssuerFeatureBlock::new(rand_nft_address().into()),
                &[21 * KEY_DATA /* Issuer */],
            ),
        ]);
    }

    #[test]
    fn dust_deposit_return_feature_block() {
        // This will change with the new dust protection in place.
        let amount = bee_message::constant::DUST_DEPOSIT_MIN;
        test_primitive(
            DustDepositReturnFeatureBlock::new(amount).unwrap(),
            &[8 * DATA /* Return Amount */],
        );
    }

    #[test]
    fn feature_block() {
        test_primitives(&[
            (
                FeatureBlock::ExpirationUnix(ExpirationUnixFeatureBlock::new(42)),
                &[
                    1 * DATA, // Block Type
                    4 * DATA, // Expiration Unix Block
                ],
            ),
            (
                FeatureBlock::ExpirationMilestoneIndex(ExpirationMilestoneIndexFeatureBlock::new(
                    rand_milestone_index(),
                )),
                &[
                    1 * DATA, // Block Type
                    4 * DATA, // Expiration Milestone Index Block
                ],
            ),
        ]);
    }
}

#[cfg(test)]
mod output {
    use super::*;

    use bee_message::output::{AliasId, NftId, OutputId, TokenId};
    use bee_test::rand::{bytes::rand_bytes_array, transaction::rand_transaction_id};

    #[test]
    fn fields() {
        // Ids
        test_primitive(AliasId::from(rand_bytes_array()), &[20 * DATA + 20 * KEY]);
        test_primitive(NftId::from(rand_bytes_array()), &[20 * DATA + 20 * KEY]);
        // TODO: Check once issues in TIP are solved.
        test_primitive(TokenId::new(rand_bytes_array()), &[38 * DATA]);
        test_primitive(OutputId::new(rand_transaction_id(), 0).unwrap(), &[34 * KEY]);
    }

    // TODO: Add test for `AliasOutput`
}

// TODO: Add tests from Hornet too.
