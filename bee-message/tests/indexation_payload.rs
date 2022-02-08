// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "cpt2")]
mod cpt2 {
    use bee_message::{payload::IndexationPayload, Error, Message};
    use bee_test::rand::bytes::rand_bytes;

    use packable::{
        bounded::{TryIntoBoundedU32Error, TryIntoBoundedU8Error},
        error::UnpackError,
        PackableExt,
    };

    #[test]
    fn kind() {
        assert_eq!(IndexationPayload::KIND, 2);
    }

    #[test]
    fn new_valid() {
        let index = rand_bytes(64);
        let data = vec![0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2];
        let indexation_data = IndexationPayload::new(index.clone(), data.clone()).unwrap();

        assert_eq!(indexation_data.index(), &index);
        assert_eq!(indexation_data.data(), &data);
    }

    #[test]
    fn new_valid_empty_data() {
        let index = rand_bytes(64);
        let data = vec![];
        let indexation_data = IndexationPayload::new(index.clone(), data.clone()).unwrap();

        assert_eq!(indexation_data.index(), &index);
        assert_eq!(indexation_data.data(), &data);
    }

    #[test]
    fn new_valid_padded() {
        let index = rand_bytes(32);
        let data = vec![];
        let indexation_data = IndexationPayload::new(index.clone(), data.clone()).unwrap();

        assert_eq!(indexation_data.index(), &index);
        assert_eq!(indexation_data.data(), &data);
    }

    #[test]
    fn new_invalid_index_length_less_than_min() {
        assert!(matches!(
            IndexationPayload::new(vec![], vec![0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2]),
            Err(Error::InvalidIndexLength(TryIntoBoundedU8Error::Invalid(0)))
        ));
    }

    #[test]
    fn new_invalid_index_length_more_than_max() {
        assert!(matches!(
            IndexationPayload::new(rand_bytes(65), vec![0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2]),
            Err(Error::InvalidIndexLength(TryIntoBoundedU8Error::Invalid(65)))
        ));
    }

    #[test]
    fn new_invalid_data_length_more_than_max() {
        assert!(matches!(
            IndexationPayload::new(rand_bytes(32), vec![0u8; Message::LENGTH_MAX + 42]),
            Err(Error::InvalidIndexationDataLength(TryIntoBoundedU32Error::Invalid(l))) if l == Message::LENGTH_MAX as u32 + 42
        ));
    }

    #[test]
    fn packed_len() {
        let indexation_data =
            IndexationPayload::new(rand_bytes(10), vec![0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2]).unwrap();

        assert_eq!(indexation_data.packed_len(), 1 + 10 + 4 + 8);
        assert_eq!(indexation_data.pack_to_vec().len(), 1 + 10 + 4 + 8);
    }

    #[test]
    fn pack_unpack_valid() {
        let indexation_data_1 =
            IndexationPayload::new(rand_bytes(32), vec![0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2]).unwrap();
        let indexation_data_2 =
            IndexationPayload::unpack_verified(&mut indexation_data_1.pack_to_vec().as_slice()).unwrap();

        assert_eq!(indexation_data_1.index(), indexation_data_2.index());
        assert_eq!(indexation_data_1.data(), indexation_data_2.data());
    }

    #[test]
    fn unpack_invalid_tag_length_less_than_min() {
        assert!(matches!(
            IndexationPayload::unpack_verified(&mut vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00].as_slice()),
            Err(UnpackError::Packable(Error::InvalidIndexLength(
                TryIntoBoundedU8Error::Invalid(0)
            )))
        ));
    }

    #[test]
    fn unpack_invalid_tag_length_more_than_max() {
        assert!(matches!(
            IndexationPayload::unpack_verified(
                &mut vec![
                    0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
                ]
                .as_slice()
            ),
            Err(UnpackError::Packable(Error::InvalidIndexLength(
                TryIntoBoundedU8Error::Invalid(65)
            )))
        ));
    }

    #[test]
    fn unpack_invalid_data_length_more_than_max() {
        assert!(matches!(
            IndexationPayload::unpack_verified(&mut vec![0x02, 0x00, 0x00, 0x35, 0x82, 0x00, 0x00].as_slice()),
            Err(UnpackError::Packable(Error::InvalidIndexationDataLength(
                TryIntoBoundedU32Error::Invalid(33333)
            )))
        ));
    }
}
