// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use packable::{Packable, PackableExt};

use crate::{bee, inx, Error};

/// Represents a type as raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raw<T: Packable> {
    data: Vec<u8>,
    _phantom: PhantomData<T>,
}

impl<T: Packable> Raw<T> {
    #[must_use]
    pub fn data(self) -> Vec<u8> {
        self.data
    }

    pub fn inner(self, visitor: &T::UnpackVisitor) -> Result<T, Error> {
        let unpacked = T::unpack_verified(self.data, visitor)
            .map_err(|e| bee_block::InxError::InvalidRawBytes(format!("{:?}", e)))?;
        Ok(unpacked)
    }

    pub fn inner_unverified(self) -> Result<T, Error> {
        let unpacked =
            T::unpack_unverified(self.data).map_err(|e| bee_block::InxError::InvalidRawBytes(format!("{:?}", e)))?;
        Ok(unpacked)
    }
}

impl<T: Packable> From<Vec<u8>> for Raw<T> {
    fn from(value: Vec<u8>) -> Self {
        Self {
            data: value,
            _phantom: PhantomData,
        }
    }
}

impl From<inx::RawOutput> for Raw<bee::Output> {
    fn from(value: inx::RawOutput) -> Self {
        value.data.into()
    }
}

impl From<Raw<bee::Output>> for inx::RawOutput {
    fn from(value: Raw<bee::Output>) -> Self {
        Self { data: value.data }
    }
}

impl From<inx::RawBlock> for Raw<bee::Block> {
    fn from(value: inx::RawBlock) -> Self {
        value.data.into()
    }
}

impl From<Raw<bee::Block>> for inx::RawBlock {
    fn from(value: Raw<bee::Block>) -> Self {
        Self { data: value.data }
    }
}

impl From<inx::RawMilestone> for Raw<bee::Payload> {
    fn from(value: inx::RawMilestone) -> Self {
        value.data.into()
    }
}

impl From<Raw<bee::Payload>> for inx::RawMilestone {
    fn from(value: Raw<bee::Payload>) -> Self {
        Self { data: value.data }
    }
}

impl From<inx::RawReceipt> for Raw<bee::MilestoneOption> {
    fn from(value: inx::RawReceipt) -> Self {
        value.data.into()
    }
}

impl From<Raw<bee::MilestoneOption>> for inx::RawReceipt {
    fn from(value: Raw<bee::MilestoneOption>) -> Self {
        Self { data: value.data }
    }
}

#[cfg(test)]
mod test {
    use bee::{rand_output, Payload};

    use super::*;
    use crate::ProtocolParameters;

    #[test]
    fn raw_output() {
        let protocol_parameters = bee::protocol_parameters();

        let output = rand_output(protocol_parameters.token_supply());

        let proto = inx::RawOutput {
            data: output.pack_to_vec(),
        };
        let raw: Raw<bee::Output> = proto.into();
        assert_eq!(output, raw.clone().inner_unverified().unwrap());
        assert_eq!(output, raw.inner(&protocol_parameters).unwrap());
    }

    #[test]
    fn raw_protocol_parameters() {
        let protocol_parameters = bee::protocol_parameters();
        let proto = inx::RawProtocolParameters::from(protocol_parameters.clone());

        let pp: ProtocolParameters = proto.into();
        assert_eq!(protocol_parameters, pp.params.inner(&()).unwrap());
    }

    #[test]
    fn raw_milestone() {
        // The `RawMilestone` field in the protobuf definitions contains a `Payload`.
        let data = vec![
            7, 0, 0, 0, 235, 183, 17, 0, 150, 184, 45, 99, 2, 126, 53, 176, 136, 103, 202, 201, 164, 84, 234, 102, 52,
            171, 19, 86, 241, 78, 148, 108, 76, 99, 18, 176, 43, 136, 175, 205, 186, 39, 155, 115, 158, 5, 27, 222, 99,
            26, 188, 240, 18, 171, 222, 80, 175, 161, 110, 80, 181, 171, 223, 86, 77, 122, 35, 69, 184, 169, 73, 177,
            144, 255, 64, 2, 125, 223, 36, 189, 63, 74, 113, 243, 26, 162, 78, 159, 68, 191, 74, 63, 138, 111, 55, 217,
            124, 187, 99, 14, 129, 112, 177, 54, 75, 51, 29, 94, 194, 108, 58, 181, 252, 101, 231, 242, 208, 69, 255,
            219, 80, 85, 132, 62, 19, 136, 1, 113, 123, 196, 54, 170, 134, 192, 96, 146, 169, 124, 108, 9, 66, 101,
            184, 243, 122, 69, 16, 194, 200, 45, 205, 89, 164, 188, 244, 218, 182, 112, 143, 192, 61, 158, 79, 230, 66,
            8, 64, 112, 65, 89, 168, 34, 147, 58, 185, 109, 59, 175, 9, 6, 150, 11, 165, 117, 104, 4, 25, 45, 224, 43,
            75, 68, 184, 151, 155, 248, 80, 131, 42, 72, 179, 204, 16, 104, 158, 232, 234, 48, 144, 225, 232, 43, 143,
            243, 228, 66, 2, 194, 2, 71, 151, 52, 184, 136, 100, 74, 7, 87, 13, 21, 233, 253, 237, 32, 38, 144, 37,
            129, 139, 141, 63, 242, 146, 133, 0, 180, 108, 136, 28, 207, 191, 37, 198, 11, 137, 29, 134, 99, 176, 132,
            59, 191, 33, 180, 34, 49, 180, 253, 241, 60, 0, 0, 0, 7, 0, 19, 204, 220, 47, 93, 61, 154, 62, 190, 6, 7,
            76, 107, 73, 180, 144, 144, 221, 121, 202, 114, 224, 74, 191, 32, 241, 15, 135, 26, 216, 41, 59, 122, 225,
            0, 114, 25, 221, 109, 248, 208, 189, 23, 229, 232, 113, 134, 209, 154, 197, 121, 222, 84, 21, 18, 147, 180,
            111, 33, 93, 249, 6, 204, 9, 26, 237, 90, 63, 46, 154, 127, 209, 143, 213, 188, 44, 179, 7, 16, 7, 34, 236,
            37, 72, 255, 227, 76, 214, 28, 226, 26, 172, 50, 134, 62, 2, 0, 69, 135, 4, 13, 224, 89, 7, 183, 8, 6, 200,
            114, 91, 218, 225, 247, 55, 7, 133, 153, 59, 42, 19, 146, 8, 226, 71, 136, 93, 78, 209, 248, 82, 246, 16,
            217, 225, 93, 30, 94, 42, 56, 146, 50, 115, 34, 65, 71, 64, 224, 194, 3, 214, 49, 48, 56, 208, 151, 197,
            57, 199, 32, 180, 93, 252, 207, 59, 34, 51, 132, 123, 206, 223, 57, 161, 194, 183, 41, 94, 140, 69, 160,
            132, 255, 227, 90, 71, 235, 62, 93, 68, 59, 220, 239, 57, 14, 0, 72, 138, 195, 251, 27, 141, 245, 239, 140,
            74, 203, 78, 241, 243, 227, 208, 57, 197, 215, 25, 125, 184, 112, 148, 166, 26, 246, 99, 32, 114, 35, 19,
            203, 209, 234, 117, 79, 52, 95, 178, 186, 163, 163, 159, 170, 181, 193, 3, 182, 201, 232, 216, 116, 93,
            226, 76, 232, 36, 89, 29, 233, 5, 148, 181, 151, 178, 220, 239, 110, 156, 86, 130, 144, 246, 74, 26, 30,
            236, 107, 221, 23, 137, 209, 176, 180, 103, 115, 225, 155, 13, 28, 244, 22, 239, 8, 13, 0, 97, 249, 95,
            237, 48, 182, 233, 191, 11, 45, 3, 147, 143, 86, 211, 87, 137, 255, 127, 14, 161, 34, 208, 28, 92, 27, 126,
            134, 149, 37, 226, 24, 56, 237, 87, 0, 183, 96, 184, 224, 155, 230, 148, 157, 39, 243, 29, 27, 81, 195,
            174, 227, 154, 43, 171, 243, 96, 112, 165, 211, 36, 106, 128, 27, 250, 221, 229, 201, 27, 196, 48, 204,
            181, 177, 52, 194, 228, 93, 199, 171, 145, 162, 168, 150, 223, 118, 5, 193, 191, 116, 67, 176, 103, 6, 144,
            6, 0, 179, 180, 201, 32, 144, 151, 32, 186, 95, 124, 48, 221, 220, 15, 145, 105, 191, 130, 67, 181, 41,
            182, 1, 252, 71, 118, 184, 203, 10, 140, 162, 83, 134, 51, 45, 102, 215, 241, 16, 125, 176, 111, 63, 214,
            168, 199, 112, 168, 105, 0, 25, 67, 255, 97, 58, 143, 219, 230, 17, 215, 200, 128, 112, 90, 220, 93, 241,
            80, 76, 206, 157, 200, 213, 240, 89, 195, 31, 8, 194, 33, 30, 18, 79, 140, 157, 224, 224, 67, 73, 172, 194,
            64, 145, 164, 118, 0, 0, 189, 237, 1, 233, 58, 223, 122, 98, 49, 24, 253, 55, 95, 217, 61, 199, 215, 221,
            242, 34, 50, 66, 57, 202, 227, 62, 78, 76, 71, 236, 59, 14, 154, 61, 180, 80, 240, 189, 219, 129, 80, 214,
            131, 79, 250, 52, 200, 162, 28, 109, 179, 218, 110, 189, 14, 147, 73, 24, 82, 10, 196, 123, 202, 106, 236,
            42, 166, 232, 18, 155, 99, 43, 173, 108, 151, 198, 155, 171, 129, 234, 233, 58, 16, 231, 104, 108, 59, 34,
            215, 202, 244, 254, 137, 121, 118, 6, 0, 241, 143, 63, 106, 45, 148, 11, 155, 172, 211, 8, 71, 19, 246,
            135, 125, 178, 32, 100, 173, 164, 51, 92, 181, 58, 225, 218, 117, 4, 79, 151, 141, 220, 110, 246, 198, 208,
            240, 129, 72, 75, 125, 143, 175, 179, 148, 34, 93, 8, 191, 115, 17, 43, 131, 229, 248, 79, 213, 224, 190,
            148, 117, 4, 49, 199, 71, 137, 238, 244, 142, 136, 193, 25, 99, 42, 171, 156, 93, 233, 59, 161, 12, 111,
            255, 59, 211, 40, 133, 187, 207, 67, 194, 150, 109, 56, 15,
        ];
        let raw = Raw::<Payload>::from(data);
        assert!(raw.inner_unverified().is_ok());
    }
}
