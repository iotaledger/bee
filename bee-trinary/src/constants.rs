pub(crate) const SIG_MSG_FRG_SIZE_TRITS: usize = 6561;
pub(crate) const SIG_MSG_FRG_SIZE_TRYTES: usize = SIG_MSG_FRG_SIZE_TRITS / 3; // 2187
pub(crate) const SIG_MSG_FRG_SIZE_BYTES: usize = SIG_MSG_FRG_SIZE_TRITS / 9 * 2; // 1458

pub(crate) const TRANSACTION_SIZE_TRITS: usize = 8019;
pub(crate) const TRANSACTION_SIZE_TRYTES: usize = TRANSACTION_SIZE_TRITS / 3; // 2673
pub(crate) const TRANSACTION_SIZE_BYTES: usize = TRANSACTION_SIZE_TRITS / 9 * 2; // 1782

pub(crate) const TRYTE_LENGTH_FOR_MAX_TOKEN_SUPPLY: usize = 11;
pub(crate) const TRYTE_LENGTH_FOR_MAX_I64: usize = 13;

pub(crate) const MAX_TRYTE_TRIPLET_ABS: i64 = 9841; // (3^9-1)/2
