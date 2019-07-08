//! IOTA protocol constants.
// Transaction (trits offset, trits length, trytes offset, trytes length, bytes offset,
// bytes length)

/// Represents a transaction field.
pub type Field = (usize, usize, usize, usize, usize, usize);

/// Offset and length of the 'signature/message fragment' field (trits, trytes, bytes)
pub const SIGNATURE_FRAGMENTS: Field = (0, 6561, 0, 2187, 0, 1458);
/// Offset and length of the 'extra data digest' field
pub const EXTRA_DATA_DIGEST: Field = (6561, 243, 2187, 81, 1458, 54);
/// Offset and length of the 'address' field
pub const ADDRESS: Field = (6804, 243, 2268, 81, 1512, 54);
/// Offset and length of the 'value' field
pub const VALUE: Field = (7047, 81, 2349, 27, 1566, 18);
/// Offset and length of the 'issuance timestamp' field
pub const ISSUANCE_TIMESTAMP: Field = (7128, 27, 2376, 9, 1584, 6);
/// Offset and length of the 'timelock_lower_bound' field
pub const TIMELOCK_LOWER_BOUND: Field = (7155, 27, 2385, 9, 1590, 6);
/// Offset and length of the 'timelock_upper_bound' field
pub const TIMELOCK_UPPER_BOUND: Field = (7182, 27, 2394, 9, 1596, 6);
/// Offset and length of the 'bundle_nonce' field
pub const BUNDLE_NONCE: Field = (7209, 81, 2403, 27, 1602, 18);
/// Offset and length of the 'trunk_hash' field
pub const TRUNK_HASH: Field = (7290, 243, 2430, 81, 1620, 54);
/// Offset and length of the 'branch_hash' field
pub const BRANCH_HASH: Field = (7533, 243, 2511, 81, 1674, 54);
/// Offset and length of the 'tag' field
pub const TAG: Field = (7776, 81, 2592, 27, 1728, 18);
/// Offset and length of the 'attachment_timestamp' field
pub const ATTACHMENT_TIMESTAMP: Field = (7857, 27, 2619, 9, 1746, 6);
/// Offset and length of the 'attachment_timestamp_lower_bound' field
pub const ATTACHMENT_TIMESTAMP_LOWER_BOUND: Field = (7884, 27, 2628, 9, 1752, 6);
/// Offset and length of the 'attachment_timestamp_upper_bound' field
pub const ATTACHMENT_TIMESTAMP_UPPER_BOUND: Field = (7911, 27, 2637, 9, 1758, 6);
/// Offset and length of the 'nonce' field
pub const NONCE: Field = (7938, 81, 2646, 27, 1764, 18);

/// Size of a transaction in trits
pub const TRANSACTION_SIZE_TRITS: usize = 8019;
/// Size of a transaction in trytes
pub const TRANSACTION_SIZE_TRYTES: usize = TRANSACTION_SIZE_TRITS / 3; // =2673
/// Size of a transaction in bytes
pub const TRANSACTION_SIZE_BYTES: usize = TRANSACTION_SIZE_TRITS / 9 * 2; // =1782

/// Size of a packet in bytes
pub const PACKET_SIZE: usize = TRANSACTION_SIZE_BYTES;

/// Maximum IOTA token supply.
pub const MAX_TOKEN_SUPPLY: i64 = (3 ^ 33) / 2 - 1;

/// Number of trytes necessary to represent the current supply of IOTA tokens ((3^33-1)/2)
pub const TRYTE_LENGTH_FOR_MAX_TOKEN_SUPPLY: usize = 11;

/// ASCII code for tryte '9'
pub const TRYTE_9_ASCII_CODE: u8 = 57;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trit_length_constants() {
        let sum = SIGNATURE_FRAGMENTS.1
            + EXTRA_DATA_DIGEST.1
            + ADDRESS.1
            + VALUE.1
            + ISSUANCE_TIMESTAMP.1
            + TIMELOCK_LOWER_BOUND.1
            + TIMELOCK_UPPER_BOUND.1
            + BUNDLE_NONCE.1
            + TRUNK_HASH.1
            + BRANCH_HASH.1
            + TAG.1
            + ATTACHMENT_TIMESTAMP.1
            + ATTACHMENT_TIMESTAMP_LOWER_BOUND.1
            + ATTACHMENT_TIMESTAMP_UPPER_BOUND.1
            + NONCE.1;

        assert_eq!(sum, TRANSACTION_SIZE_TRITS);
    }

    #[test]
    fn trit_offset_constants() {
        assert_eq!(EXTRA_DATA_DIGEST.0, SIGNATURE_FRAGMENTS.0 + SIGNATURE_FRAGMENTS.1);
        assert_eq!(ADDRESS.0, EXTRA_DATA_DIGEST.0 + EXTRA_DATA_DIGEST.1);
        assert_eq!(VALUE.0, ADDRESS.0 + ADDRESS.1);
        assert_eq!(ISSUANCE_TIMESTAMP.0, VALUE.0 + VALUE.1);
        assert_eq!(TIMELOCK_LOWER_BOUND.0, ISSUANCE_TIMESTAMP.0 + ISSUANCE_TIMESTAMP.1);
        assert_eq!(
            TIMELOCK_UPPER_BOUND.0,
            TIMELOCK_LOWER_BOUND.0 + TIMELOCK_LOWER_BOUND.1
        );
        assert_eq!(BUNDLE_NONCE.0, TIMELOCK_UPPER_BOUND.0 + TIMELOCK_UPPER_BOUND.1);
        assert_eq!(TRUNK_HASH.0, BUNDLE_NONCE.0 + BUNDLE_NONCE.1);
        assert_eq!(BRANCH_HASH.0, TRUNK_HASH.0 + TRUNK_HASH.1);
        assert_eq!(TAG.0, BRANCH_HASH.0 + BRANCH_HASH.1);
        assert_eq!(ATTACHMENT_TIMESTAMP.0, TAG.0 + TAG.1);
        assert_eq!(
            ATTACHMENT_TIMESTAMP_LOWER_BOUND.0,
            ATTACHMENT_TIMESTAMP.0 + ATTACHMENT_TIMESTAMP.1
        );
        assert_eq!(
            ATTACHMENT_TIMESTAMP_UPPER_BOUND.0,
            ATTACHMENT_TIMESTAMP_LOWER_BOUND.0 + ATTACHMENT_TIMESTAMP_LOWER_BOUND.1
        );
        assert_eq!(
            NONCE.0,
            ATTACHMENT_TIMESTAMP_UPPER_BOUND.0 + ATTACHMENT_TIMESTAMP_UPPER_BOUND.1
        );
    }

    #[test]
    fn tryte_length_constants() {
        let sum = SIGNATURE_FRAGMENTS.3
            + EXTRA_DATA_DIGEST.3
            + ADDRESS.3
            + VALUE.3
            + ISSUANCE_TIMESTAMP.3
            + TIMELOCK_LOWER_BOUND.3
            + TIMELOCK_UPPER_BOUND.3
            + BUNDLE_NONCE.3
            + TRUNK_HASH.3
            + BRANCH_HASH.3
            + TAG.3
            + ATTACHMENT_TIMESTAMP.3
            + ATTACHMENT_TIMESTAMP_LOWER_BOUND.3
            + ATTACHMENT_TIMESTAMP_UPPER_BOUND.3
            + NONCE.3;

        assert_eq!(sum, TRANSACTION_SIZE_TRYTES);
    }

    #[test]
    fn tryte_offset_constants() {
        assert_eq!(SIGNATURE_FRAGMENTS.0 / 3, SIGNATURE_FRAGMENTS.2);
        assert_eq!(EXTRA_DATA_DIGEST.0 / 3, EXTRA_DATA_DIGEST.2);
        assert_eq!(ADDRESS.0 / 3, ADDRESS.2);
        assert_eq!(VALUE.0 / 3, VALUE.2);
        assert_eq!(ISSUANCE_TIMESTAMP.0 / 3, ISSUANCE_TIMESTAMP.2);
        assert_eq!(TIMELOCK_LOWER_BOUND.0 / 3, TIMELOCK_LOWER_BOUND.2);
        assert_eq!(TIMELOCK_UPPER_BOUND.0 / 3, TIMELOCK_UPPER_BOUND.2);
        assert_eq!(BUNDLE_NONCE.0 / 3, BUNDLE_NONCE.2);
        assert_eq!(TRUNK_HASH.0 / 3, TRUNK_HASH.2);
        assert_eq!(BRANCH_HASH.0 / 3, BRANCH_HASH.2);
        assert_eq!(TAG.0 / 3, TAG.2);
        assert_eq!(ATTACHMENT_TIMESTAMP.0 / 3, ATTACHMENT_TIMESTAMP.2);
        assert_eq!(
            ATTACHMENT_TIMESTAMP_LOWER_BOUND.0 / 3,
            ATTACHMENT_TIMESTAMP_LOWER_BOUND.2
        );
        assert_eq!(
            ATTACHMENT_TIMESTAMP_UPPER_BOUND.0 / 3,
            ATTACHMENT_TIMESTAMP_UPPER_BOUND.2
        );
        assert_eq!(NONCE.0 / 3, NONCE.2);
    }

    #[test]
    fn byte_length_constants() {
        let sum = SIGNATURE_FRAGMENTS.5
            + EXTRA_DATA_DIGEST.5
            + ADDRESS.5
            + VALUE.5
            + ISSUANCE_TIMESTAMP.5
            + TIMELOCK_LOWER_BOUND.5
            + TIMELOCK_UPPER_BOUND.5
            + BUNDLE_NONCE.5
            + TRUNK_HASH.5
            + BRANCH_HASH.5
            + TAG.5
            + ATTACHMENT_TIMESTAMP.5
            + ATTACHMENT_TIMESTAMP_LOWER_BOUND.5
            + ATTACHMENT_TIMESTAMP_UPPER_BOUND.5
            + NONCE.5;

        assert_eq!(sum, TRANSACTION_SIZE_BYTES);
    }

    #[test]
    fn byte_offset_constants() {
        assert_eq!(SIGNATURE_FRAGMENTS.2 / 3 * 2, SIGNATURE_FRAGMENTS.4);
        assert_eq!(EXTRA_DATA_DIGEST.2 / 3 * 2, EXTRA_DATA_DIGEST.4);
        assert_eq!(ADDRESS.2 / 3 * 2, ADDRESS.4);
        assert_eq!(VALUE.2 / 3 * 2, VALUE.4);
        assert_eq!(ISSUANCE_TIMESTAMP.2 / 3 * 2, ISSUANCE_TIMESTAMP.4);
        assert_eq!(TIMELOCK_LOWER_BOUND.2 / 3 * 2, TIMELOCK_LOWER_BOUND.4);
        assert_eq!(TIMELOCK_UPPER_BOUND.2 / 3 * 2, TIMELOCK_UPPER_BOUND.4);
        assert_eq!(BUNDLE_NONCE.2 / 3 * 2, BUNDLE_NONCE.4);
        assert_eq!(TRUNK_HASH.2 / 3 * 2, TRUNK_HASH.4);
        assert_eq!(BRANCH_HASH.2 / 3 * 2, BRANCH_HASH.4);
        assert_eq!(TAG.2 / 3 * 2, TAG.4);
        assert_eq!(ATTACHMENT_TIMESTAMP.2 / 3 * 2, ATTACHMENT_TIMESTAMP.4);
        assert_eq!(
            ATTACHMENT_TIMESTAMP_LOWER_BOUND.2 / 3 * 2,
            ATTACHMENT_TIMESTAMP_LOWER_BOUND.4
        );
        assert_eq!(
            ATTACHMENT_TIMESTAMP_UPPER_BOUND.2 / 3 * 2,
            ATTACHMENT_TIMESTAMP_UPPER_BOUND.4
        );
        assert_eq!(NONCE.2 / 3 * 2, NONCE.4);
    }
}
