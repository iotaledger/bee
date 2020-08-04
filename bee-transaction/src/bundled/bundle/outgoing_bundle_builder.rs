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

use crate::bundled::{
    constants::{IOTA_SUPPLY, PAYLOAD_TRIT_LEN},
    Address, Bundle, BundledTransactionBuilder, BundledTransactionBuilders, BundledTransactionError,
    BundledTransactionField, BundledTransactions, Index, Payload, Tag,
};

use bee_crypto::ternary::{
    sponge::{Kerl, Sponge},
    Hash,
};
use bee_signing::ternary::{
    seed::Seed,
    wots::{normalize, WotsSecurityLevel, WotsSpongePrivateKeyGeneratorBuilder},
    PrivateKey, PrivateKeyGenerator, Signature,
};
use bee_ternary::Btrit;

use std::marker::PhantomData;

#[derive(Debug)]
pub enum OutgoingBundleBuilderError {
    Empty,
    UnsignedInput,
    InvalidValue(i64),
    MissingTransactionBuilderField(&'static str),
    TransactionError(BundledTransactionError),
    FailedSigningOperation,
}

pub trait OutgoingBundleBuilderStage {}

pub struct OutgoingRaw;
impl OutgoingBundleBuilderStage for OutgoingRaw {}

pub struct OutgoingSealed;
impl OutgoingBundleBuilderStage for OutgoingSealed {}

pub struct OutgoingSigned;
impl OutgoingBundleBuilderStage for OutgoingSigned {}

pub struct OutgoingAttached;
impl OutgoingBundleBuilderStage for OutgoingAttached {}

pub struct StagedOutgoingBundleBuilder<E, S> {
    builders: BundledTransactionBuilders,
    essence_sponge: PhantomData<E>,
    stage: PhantomData<S>,
}

// TODO default to Kerl
pub type OutgoingBundleBuilder = StagedOutgoingBundleBuilder<Kerl, OutgoingRaw>;

impl<E, S> StagedOutgoingBundleBuilder<E, S>
where
    E: Sponge + Default,
    S: OutgoingBundleBuilderStage,
{
    // TODO TEST
    fn calculate_bundle_hash(&mut self) -> Result<(), OutgoingBundleBuilderError> {
        let mut sponge = E::default();
        let mut obsolete_tag = match self.builders.0.get(0) {
            Some(builder) => match builder.obsolete_tag.as_ref() {
                Some(obsolete_tag) => obsolete_tag.to_inner().to_owned(),
                _ => {
                    return Err(OutgoingBundleBuilderError::MissingTransactionBuilderField(
                        "obsolete_tag",
                    ))
                }
            },
            _ => return Err(OutgoingBundleBuilderError::Empty),
        };

        let hash = loop {
            sponge.reset();

            for builder in &self.builders.0 {
                let _ = sponge.absorb(&builder.essence());
            }

            // TODO squeeze into
            let hash = sponge
                .squeeze()
                .unwrap_or_else(|_| panic!("Panicked when unwrapping the sponge hash function."));

            let mut has_m_bug = false;
            // Safe to unwrap because we know `hash` has a valid size since it's squeezed from the sponge.
            for trits in normalize(&hash).unwrap().chunks(3) {
                let mut is_m = true;

                for trit in trits.iter() {
                    if trit != Btrit::PlusOne {
                        is_m = false;
                        break;
                    }
                }

                if is_m {
                    has_m_bug = true;
                    break;
                }
            }

            if !has_m_bug {
                break Hash::from_inner_unchecked(hash);
            } else {
                // obsolete_tag + 1
                // TODO we may want to move this operation to the ternary crate
                for i in 0..obsolete_tag.len() {
                    // Safe to unwrap since it's in the range of tag
                    match obsolete_tag.get(i).unwrap() {
                        Btrit::NegOne => {
                            obsolete_tag.set(i, Btrit::Zero);
                            break;
                        }
                        Btrit::Zero => {
                            obsolete_tag.set(i, Btrit::PlusOne);
                            break;
                        }
                        Btrit::PlusOne => obsolete_tag.set(i, Btrit::NegOne),
                    };
                }
                // Safe to unwrap because we already check first tx exists.
                self.builders
                    .0
                    .get_mut(0)
                    .unwrap()
                    .obsolete_tag
                    .replace(Tag::from_inner_unchecked(obsolete_tag.clone()));
            }
        };

        for builder in &mut self.builders.0 {
            builder.bundle.replace(hash.clone());
        }

        Ok(())
    }
}

impl<E: Sponge + Default> StagedOutgoingBundleBuilder<E, OutgoingRaw> {
    // TODO TEST
    pub fn new() -> Self {
        Self {
            builders: BundledTransactionBuilders::default(),
            essence_sponge: PhantomData,
            stage: PhantomData,
        }
    }

    // TODO TEST
    pub fn push(&mut self, builder: BundledTransactionBuilder) {
        self.builders.push(builder);
    }

    // TODO TEST
    pub fn seal(mut self) -> Result<StagedOutgoingBundleBuilder<E, OutgoingSealed>, OutgoingBundleBuilderError> {
        // TODO Impl
        // TODO should call validate() on transaction builders ?
        let mut sum: i64 = 0;
        let last_index = self.builders.len() - 1;

        if self.builders.len() == 0 {
            return Err(OutgoingBundleBuilderError::Empty);
        }

        for (index, builder) in self.builders.0.iter_mut().enumerate() {
            if builder.payload.is_none() {
                return Err(OutgoingBundleBuilderError::MissingTransactionBuilderField("payload"));
            } else if builder.address.is_none() {
                return Err(OutgoingBundleBuilderError::MissingTransactionBuilderField("address"));
            } else if builder.value.is_none() {
                return Err(OutgoingBundleBuilderError::MissingTransactionBuilderField("value"));
            } else if builder.tag.is_none() {
                return Err(OutgoingBundleBuilderError::MissingTransactionBuilderField("tag"));
            }

            builder.index.replace(Index::from_inner_unchecked(index));
            builder.last_index.replace(Index::from_inner_unchecked(last_index));

            // Safe to unwrap since we just checked it's not None
            sum += builder.value.as_ref().unwrap().to_inner();
            if sum.abs() > IOTA_SUPPLY {
                return Err(OutgoingBundleBuilderError::InvalidValue(sum));
            }
        }

        if sum != 0 {
            return Err(OutgoingBundleBuilderError::InvalidValue(sum));
        }

        self.calculate_bundle_hash()?;

        Ok(StagedOutgoingBundleBuilder::<E, OutgoingSealed> {
            builders: self.builders,
            essence_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E: Sponge + Default> StagedOutgoingBundleBuilder<E, OutgoingSealed> {
    // TODO TEST
    fn has_no_input(&self) -> Result<(), OutgoingBundleBuilderError> {
        for builder in &self.builders.0 {
            // Safe to unwrap since we made sure it's not None in `seal`
            if *builder.value.as_ref().unwrap().to_inner() < 0 {
                return Err(OutgoingBundleBuilderError::UnsignedInput);
            }
        }

        Ok(())
    }

    // TODO TEST
    pub fn attach_local(
        self,
        trunk: Hash,
        branch: Hash,
    ) -> Result<StagedOutgoingBundleBuilder<E, OutgoingAttached>, OutgoingBundleBuilderError> {
        // Checking that no transaction actually needs to be signed (no inputs)
        self.has_no_input()?;

        StagedOutgoingBundleBuilder::<E, OutgoingSigned> {
            builders: self.builders,
            essence_sponge: PhantomData,
            stage: PhantomData,
        }
        .attach_local(trunk, branch)
    }

    // TODO TEST
    pub fn attach_remote(
        self,
        trunk: Hash,
        branch: Hash,
    ) -> Result<StagedOutgoingBundleBuilder<E, OutgoingAttached>, OutgoingBundleBuilderError> {
        // Checking that no transaction actually needs to be signed (no inputs)
        self.has_no_input()?;

        StagedOutgoingBundleBuilder::<E, OutgoingSigned> {
            builders: self.builders,
            essence_sponge: PhantomData,
            stage: PhantomData,
        }
        .attach_remote(trunk, branch)
    }

    // TODO TEST
    // TODO Right now this method receive inputs have same order as address in bundle.
    // We probably want to check it is the right input for the address.
    pub fn sign(
        mut self,
        seed: &Seed,
        inputs: &[(u64, Address, WotsSecurityLevel)],
    ) -> Result<StagedOutgoingBundleBuilder<E, OutgoingSigned>, OutgoingBundleBuilderError> {
        // Safe to unwrap `get` because bundle is sealed.
        // Safe to unwrap `normalize` because we know the bundle hash has a valid size.
        let message = normalize(self.builders.0.get(0).unwrap().bundle.as_ref().unwrap().to_inner()).unwrap();

        let mut signature_fragments: Vec<Payload> = Vec::new();

        for (index, _, security) in inputs {
            let key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
                .with_security_level(*security)
                .build()
                // Safe to unwrap because security level is provided
                .unwrap();
            // Create subseed and then sign the message
            let signature = key_generator
                .generate_from_seed(seed, *index)
                .map_err(|_| OutgoingBundleBuilderError::FailedSigningOperation)?
                .sign(&message)
                .map_err(|_| OutgoingBundleBuilderError::FailedSigningOperation)?;

            // Split signature into fragments
            for fragment in signature.as_trits().chunks(PAYLOAD_TRIT_LEN) {
                signature_fragments.push(Payload::from_inner_unchecked(fragment.to_owned()));
            }
        }

        // Find the first input tx
        let mut input_index = 0;
        for i in &self.builders.0 {
            if i.value.as_ref().unwrap().to_inner() < &0 {
                input_index = i.index.as_ref().unwrap().to_inner().to_owned();
                break;
            }
        }

        // We assume input tx are placed in order and have correct amount based on security level
        for payload in signature_fragments {
            let builder = self.builders.0.get_mut(input_index).unwrap();
            builder.payload = Some(payload);
            input_index += 1;
        }

        Ok(StagedOutgoingBundleBuilder::<E, OutgoingSigned> {
            builders: self.builders,
            essence_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E: Sponge + Default> StagedOutgoingBundleBuilder<E, OutgoingSigned> {
    // TODO TEST
    pub fn attach_local(
        self,
        _trunk: Hash,
        _branch: Hash,
    ) -> Result<StagedOutgoingBundleBuilder<E, OutgoingAttached>, OutgoingBundleBuilderError> {
        // TODO Impl
        Ok(StagedOutgoingBundleBuilder::<E, OutgoingAttached> {
            builders: self.builders,
            essence_sponge: PhantomData,
            stage: PhantomData,
        })
    }

    // TODO TEST
    pub fn attach_remote(
        self,
        _trunk: Hash,
        _branch: Hash,
    ) -> Result<StagedOutgoingBundleBuilder<E, OutgoingAttached>, OutgoingBundleBuilderError> {
        // TODO Impl
        Ok(StagedOutgoingBundleBuilder::<E, OutgoingAttached> {
            builders: self.builders,
            essence_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E: Sponge + Default> StagedOutgoingBundleBuilder<E, OutgoingAttached> {
    // TODO TEST
    pub fn build(self) -> Result<Bundle, OutgoingBundleBuilderError> {
        let mut transactions = BundledTransactions::new();

        for transaction_builder in self.builders.0 {
            transactions.push(
                transaction_builder
                    .build()
                    .map_err(|e| OutgoingBundleBuilderError::TransactionError(e))?,
            );
        }

        Ok(Bundle(transactions))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::bundled::{Address, Nonce, Payload, Tag, Timestamp, Value};

    use bee_signing::ternary::{seed::Seed, wots::WotsSignature, PublicKey, RecoverableSignature};
    use bee_ternary::{T1B1Buf, TritBuf};

    fn default_transaction_builder(index: usize, last_index: usize) -> BundledTransactionBuilder {
        BundledTransactionBuilder::new()
            .with_payload(Payload::zeros())
            .with_address(Address::zeros())
            .with_value(Value::from_inner_unchecked(0))
            .with_obsolete_tag(Tag::zeros())
            .with_timestamp(Timestamp::from_inner_unchecked(0))
            .with_index(Index::from_inner_unchecked(index))
            .with_last_index(Index::from_inner_unchecked(last_index))
            .with_tag(Tag::zeros())
            .with_attachment_ts(Timestamp::from_inner_unchecked(0))
            .with_bundle(Hash::zeros())
            .with_trunk(Hash::zeros())
            .with_branch(Hash::zeros())
            .with_attachment_lbts(Timestamp::from_inner_unchecked(0))
            .with_attachment_ubts(Timestamp::from_inner_unchecked(0))
            .with_nonce(Nonce::zeros())
    }

    fn bundle_builder_signature_check(security: WotsSecurityLevel) -> Result<(), OutgoingBundleBuilderError> {
        let bundle_size = 4;
        let mut bundle_builder = OutgoingBundleBuilder::new();
        let seed = Seed::rand();
        let privkey = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
            .with_security_level(security)
            .build()
            .unwrap()
            .generate_from_seed(&seed, 0)
            .unwrap();
        let address = Address::from_inner_unchecked(privkey.generate_public_key().unwrap().as_trits().to_owned());

        // Transfer
        bundle_builder.push(default_transaction_builder(0, bundle_size - 1).with_value(Value::from_inner_unchecked(1)));

        // Input
        bundle_builder.push(
            default_transaction_builder(1, bundle_size - 1)
                .with_address(address.clone())
                .with_value(Value::from_inner_unchecked(-1)),
        );
        bundle_builder.push(default_transaction_builder(2, bundle_size - 1).with_address(address.clone()));
        bundle_builder.push(default_transaction_builder(3, bundle_size - 1).with_address(address.clone()));

        // Build bundle and sign
        let bundle = bundle_builder
            .seal()?
            .sign(&seed, &[(0, address.clone(), security)])?
            .attach_local(Hash::zeros(), Hash::zeros())?
            .build()?;
        assert_eq!(bundle.len(), bundle_size);

        // Validate signature
        let security = match security {
            WotsSecurityLevel::Low => 1,
            WotsSecurityLevel::Medium => 2,
            WotsSecurityLevel::High => 3,
        };
        let mut signature = TritBuf::<T1B1Buf>::zeros(PAYLOAD_TRIT_LEN * security);
        let mut offset = 0;
        for i in 1..security + 1 {
            let input = bundle.0.get(i).unwrap();
            signature[offset..][..PAYLOAD_TRIT_LEN].copy_from(input.payload.to_inner());

            offset += PAYLOAD_TRIT_LEN;
        }
        let res = WotsSignature::<Kerl>::from_trits(signature)
            .unwrap()
            .recover_public_key(&normalize(bundle.0.get(1).unwrap().bundle.to_inner()).unwrap())
            .unwrap();
        assert_eq!(address.to_inner(), res.as_trits());

        Ok(())
    }

    fn bundle_builder_different_security_check() -> Result<(), OutgoingBundleBuilderError> {
        let bundle_size = 4;
        let mut bundle_builder = OutgoingBundleBuilder::new();
        let seed = Seed::rand();
        let address_low = Address::from_inner_unchecked(
            WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
                .with_security_level(WotsSecurityLevel::Low)
                .build()
                .unwrap()
                .generate_from_seed(&seed, 0)
                .unwrap()
                .generate_public_key()
                .unwrap()
                .as_trits()
                .to_owned(),
        );
        let address_medium = Address::from_inner_unchecked(
            WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
                .with_security_level(WotsSecurityLevel::Medium)
                .build()
                .unwrap()
                .generate_from_seed(&seed, 1)
                .unwrap()
                .generate_public_key()
                .unwrap()
                .as_trits()
                .to_owned(),
        );

        // Transfer
        bundle_builder.push(default_transaction_builder(0, bundle_size - 1).with_value(Value::from_inner_unchecked(2)));

        // Input
        bundle_builder.push(
            default_transaction_builder(1, bundle_size - 1)
                .with_address(address_low.clone())
                .with_value(Value::from_inner_unchecked(-1)),
        );
        bundle_builder.push(
            default_transaction_builder(2, bundle_size - 1)
                .with_address(address_medium.clone())
                .with_value(Value::from_inner_unchecked(-1)),
        );
        bundle_builder.push(default_transaction_builder(3, bundle_size - 1).with_address(address_medium.clone()));

        // Build bundle and sign
        let bundle = bundle_builder
            .seal()?
            .sign(
                &seed,
                &[
                    (0, address_low.clone(), WotsSecurityLevel::Low),
                    (1, address_medium.clone(), WotsSecurityLevel::Medium),
                ],
            )?
            .attach_local(Hash::zeros(), Hash::zeros())?
            .build()?;
        assert_eq!(bundle.len(), bundle_size);

        // Validate signature
        let res_low = WotsSignature::<Kerl>::from_trits(bundle.0.get(1).unwrap().payload.to_inner().to_owned())
            .unwrap()
            .recover_public_key(&normalize(bundle.0.get(1).unwrap().bundle.to_inner()).unwrap())
            .unwrap();
        assert_eq!(address_low.to_inner(), res_low.as_trits());

        let mut signature = TritBuf::<T1B1Buf>::zeros(PAYLOAD_TRIT_LEN * 2);
        let mut offset = 0;
        for i in 2..4 {
            let input = bundle.0.get(i).unwrap();
            signature[offset..][..PAYLOAD_TRIT_LEN].copy_from(input.payload.to_inner());
            offset += PAYLOAD_TRIT_LEN;
        }
        let res_medium = WotsSignature::<Kerl>::from_trits(signature)
            .unwrap()
            .recover_public_key(&normalize(bundle.0.get(2).unwrap().bundle.to_inner()).unwrap())
            .unwrap();
        assert_eq!(address_medium.to_inner(), res_medium.as_trits());

        Ok(())
    }

    // TODO Also check to attach if value ?
    #[test]
    fn outgoing_bundle_builder_value_test() -> Result<(), OutgoingBundleBuilderError> {
        // Check each security
        bundle_builder_signature_check(WotsSecurityLevel::Low)?;
        bundle_builder_signature_check(WotsSecurityLevel::Medium)?;
        bundle_builder_signature_check(WotsSecurityLevel::High)?;
        // Check inputs have different security
        bundle_builder_different_security_check()
    }

    // TODO Also check to sign if data ?
    #[test]
    fn outgoing_bundle_builder_data_test() -> Result<(), OutgoingBundleBuilderError> {
        let bundle_size = 3;
        let mut bundle_builder = OutgoingBundleBuilder::new();

        for i in 0..bundle_size {
            bundle_builder.push(default_transaction_builder(i, bundle_size - 1));
        }

        let bundle = bundle_builder
            .seal()?
            .attach_local(Hash::zeros(), Hash::zeros())?
            .build()?;

        assert_eq!(bundle.len(), bundle_size);

        Ok(())
    }
}
