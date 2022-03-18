// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "cpt2")]
mod cpt2 {
    use bee_message::{
        address::{Address, Ed25519Address},
        constant::IOTA_SUPPLY,
        output::{Output, SignatureLockedSingleOutput},
        Error,
    };

    use packable::{bounded::InvalidBoundedU64, error::UnpackError, PackableExt};

    use std::str::FromStr;

    const ED25519_ADDRESS: &str = "0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

    #[test]
    fn kind() {
        assert_eq!(SignatureLockedSingleOutput::KIND, 0);
    }

    #[test]
    fn new_valid_min_amount() {
        assert_eq!(
            SignatureLockedSingleOutput::new(Ed25519Address::from_str(ED25519_ADDRESS).unwrap().into(), 1)
                .unwrap()
                .amount(),
            *Output::AMOUNT_RANGE.start()
        );
    }

    #[test]
    fn new_valid_max_amount() {
        assert_eq!(
            SignatureLockedSingleOutput::new(Ed25519Address::from_str(ED25519_ADDRESS).unwrap().into(), IOTA_SUPPLY)
                .unwrap()
                .amount(),
            *Output::AMOUNT_RANGE.end()
        );
    }

    #[test]
    fn new_invalid_less_than_min_amount() {
        assert!(matches!(
            SignatureLockedSingleOutput::new(Ed25519Address::from_str(ED25519_ADDRESS).unwrap().into(), 0),
            Err(Error::InvalidOutputAmount(InvalidBoundedU64(0)))
        ));
    }

    #[test]
    fn new_invalid_more_than_max_amount() {
        assert!(matches!(
            SignatureLockedSingleOutput::new(
                Ed25519Address::from_str(ED25519_ADDRESS).unwrap().into(),
                3_038_287_259_199_220_266
            ),
            Err(Error::InvalidOutputAmount(InvalidBoundedU64(3_038_287_259_199_220_266)))
        ));
    }

    #[test]
    fn packed_len() {
        let output =
            SignatureLockedSingleOutput::new(Ed25519Address::from_str(ED25519_ADDRESS).unwrap().into(), 1000).unwrap();

        assert_eq!(output.packed_len(), 41);
        assert_eq!(output.pack_to_vec().len(), 41);
    }

    #[test]
    fn pack_unpack_valid() {
        let output_1 =
            SignatureLockedSingleOutput::new(Ed25519Address::from_str(ED25519_ADDRESS).unwrap().into(), 1000).unwrap();
        let output_2 = SignatureLockedSingleOutput::unpack_verified(&mut output_1.pack_to_vec().as_slice()).unwrap();

        assert_eq!(output_1, output_2);
    }

    #[test]
    fn pack_unpack_invalid() {
        let mut bytes = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()).pack_to_vec();
        bytes.extend([0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a]);
        assert!(matches!(
            SignatureLockedSingleOutput::unpack_verified(&mut bytes.as_slice()),
            Err(UnpackError::Packable(Error::InvalidOutputAmount(InvalidBoundedU64(
                3_038_287_259_199_220_266
            ))))
        ));
    }
}
