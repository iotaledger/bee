//! IOTA transaction definition.

use crate::constants::*;
use crate::time;

use bee_trinary::bytes;
use bee_trinary::numbers;
use bee_trinary::trits;
use bee_trinary::tryte_strings;
use bee_trinary::trytes;
use bee_trinary::util;

/// As bytes serialized transaction.
pub type TransactionBytes = [u8; TRANSACTION_SIZE_BYTES];

/// As trytes serialized transaction.
pub type TransactionTrytes = [u8; TRANSACTION_SIZE_TRYTES];

/// As trits serialized transaction.
pub type TransactionTrits = [i8; TRANSACTION_SIZE_TRITS];

/// Represents a decoded Bee transaction.
#[derive(Clone, Debug)]
pub struct Transaction {
    /// Data field.
    pub signature_fragments: String, // [char; 2187]
    /// Extra data digest field.
    pub extra_data_digest: String, // [char; 81]
    /// Address field.
    pub address: String, // [char; 81]
    /// Value field.
    pub value: i64,
    /// Issuance timestamp field.
    pub issuance_timestamp: i64, // u64
    /// Timelock lower bound field.
    pub timelock_lower_bound: i64, // u64
    /// Timelock upper bound field.
    pub timelock_upper_bound: i64, //u64
    /// Bundle nonce field.
    pub bundle_nonce: String, // [char; 27]
    /// Trunk hash field.
    pub trunk: String, // [char; 81]
    /// Branch hash field.
    pub branch: String, // [char; 81]
    /// Tag field.
    pub tag: String, // [char; 27]
    /// Attachment timestamp field.
    pub attachment_timestamp: i64, // u64
    /// Attachment timestamp lower bound field.
    pub attachment_timestamp_lower_bound: i64, //u64
    /// Attachment timestamp upper bound field.
    pub attachment_timestamp_upper_bound: i64, //u64
    /// Nonce field.
    pub nonce: String, // [char; 27]
}

impl Transaction {
    /// Deserializes a transaction from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Transaction::from_trytes(&trytes::from_bytes_all(bytes))
    }

    /// Deserializes a transaction from a tryte string.
    pub fn from_tryte_str(tryte_str: &str) -> Self {
        assert!(util::is_tryte_str(tryte_str));

        let bytes = tryte_str.as_bytes();
        assert_eq!(bytes.len(), TRANSACTION_SIZE_TRYTES);

        let mut trytes = [0; TRANSACTION_SIZE_TRYTES];
        trytes[..].copy_from_slice(&bytes[..]);

        Transaction::from_trytes(&trytes)
    }

    /// Deserializes a transaction from trytes.
    pub fn from_trytes(trytes: &[u8]) -> Self {
        let signature_fragments = tryte_strings::from_trytes(
            &trytes[SIGNATURE_FRAGMENTS.2..EXTRA_DATA_DIGEST.2],
        );

        let extra_data_digest =
            tryte_strings::from_trytes(&trytes[EXTRA_DATA_DIGEST.2..ADDRESS.2]);

        let address = tryte_strings::from_trytes(&trytes[ADDRESS.2..VALUE.2]);

        // NOTE: only correct until token supply increases, then upper_bound =
        // ISSUANCE_TIMESTAMP.2
        let value = numbers::from_trytes_max11(
            &trytes[VALUE.2..VALUE.2 + TRYTE_LENGTH_FOR_MAX_TOKEN_SUPPLY],
        );

        let issuance_timestamp = numbers::from_trytes_max11(
            &trytes[ISSUANCE_TIMESTAMP.2..TIMELOCK_LOWER_BOUND.2],
        );

        let timelock_lower_bound = numbers::from_trytes_max11(
            &trytes[TIMELOCK_LOWER_BOUND.2..TIMELOCK_UPPER_BOUND.2],
        );

        let timelock_upper_bound =
            numbers::from_trytes_max11(&trytes[TIMELOCK_UPPER_BOUND.2..BUNDLE_NONCE.2]);

        let bundle_nonce =
            tryte_strings::from_trytes(&trytes[BUNDLE_NONCE.2..TRUNK_HASH.2]);

        let trunk = tryte_strings::from_trytes(&trytes[TRUNK_HASH.2..BRANCH_HASH.2]);

        let branch = tryte_strings::from_trytes(&trytes[BRANCH_HASH.2..TAG.2]);

        let tag = tryte_strings::from_trytes(&trytes[TAG.2..ATTACHMENT_TIMESTAMP.2]);

        let attachment_timestamp = numbers::from_trytes_max11(
            &trytes[ATTACHMENT_TIMESTAMP.2..ATTACHMENT_TIMESTAMP_LOWER_BOUND.2],
        );

        let attachment_timestamp_lower_bound = numbers::from_trytes_max11(
            &trytes
                [ATTACHMENT_TIMESTAMP_LOWER_BOUND.2..ATTACHMENT_TIMESTAMP_UPPER_BOUND.2],
        );

        let attachment_timestamp_upper_bound = numbers::from_trytes_max11(
            &trytes[ATTACHMENT_TIMESTAMP_UPPER_BOUND.2..NONCE.2],
        );

        let nonce = tryte_strings::from_trytes(&trytes[NONCE.2..TRANSACTION_SIZE_TRYTES]);

        Transaction {
            signature_fragments,
            extra_data_digest,
            address,
            value,
            issuance_timestamp,
            timelock_lower_bound,
            timelock_upper_bound,
            bundle_nonce,
            trunk,
            branch,
            tag,
            attachment_timestamp,
            attachment_timestamp_lower_bound,
            attachment_timestamp_upper_bound,
            nonce,
        }
    }

    /// Serializes this transaction to bytes.
    pub fn as_bytes(&self) -> TransactionBytes {
        bytes::from_trytes_all(&self.as_trytes())
    }

    /// Serializes this transaction to a tryte string.
    pub fn as_tryte_string(&self) -> String {
        tryte_strings::from_trytes(&self.as_trytes())
    }

    /// Serializes this transaction to trits.
    pub fn as_trits(&self) -> TransactionTrits {
        trits::from_trytes_all(&self.as_trytes())
    }

    /// Serializes this transaction to trytes.
    pub fn as_trytes(&self) -> TransactionTrytes {
        let mut trytes = [TRYTE_9_ASCII_CODE; TRANSACTION_SIZE_TRYTES];

        trytes[SIGNATURE_FRAGMENTS.2..EXTRA_DATA_DIGEST.2].copy_from_slice(
            &trytes::from_tryte_str(&self.signature_fragments)[..SIGNATURE_FRAGMENTS.3],
        );

        trytes[EXTRA_DATA_DIGEST.2..ADDRESS.2]
            .copy_from_slice(&self.extra_data_digest.as_bytes()[..EXTRA_DATA_DIGEST.3]);

        trytes[ADDRESS.2..VALUE.2].copy_from_slice(&self.address.as_bytes()[..ADDRESS.3]);

        // NOTE: only correct until token supply increases, then upper_bound =
        // ISSUANCE_TIMESTAMP.2
        trytes[VALUE.2..VALUE.2 + TRYTE_LENGTH_FOR_MAX_TOKEN_SUPPLY]
            .copy_from_slice(&trytes::from_num_i64_to_11(self.value));

        trytes[ISSUANCE_TIMESTAMP.2..TIMELOCK_LOWER_BOUND.2]
            .copy_from_slice(&trytes::from_num_i64_to_9(self.issuance_timestamp));

        trytes[TIMELOCK_LOWER_BOUND.2..TIMELOCK_UPPER_BOUND.2]
            .copy_from_slice(&trytes::from_num_i64_to_9(self.timelock_lower_bound));

        trytes[TIMELOCK_UPPER_BOUND.2..BUNDLE_NONCE.2]
            .copy_from_slice(&trytes::from_num_i64_to_9(self.timelock_upper_bound));

        trytes[BUNDLE_NONCE.2..TRUNK_HASH.2]
            .copy_from_slice(&self.bundle_nonce.as_bytes()[..BUNDLE_NONCE.3]);

        trytes[TRUNK_HASH.2..BRANCH_HASH.2]
            .copy_from_slice(&self.trunk.as_bytes()[..TRUNK_HASH.3]);

        trytes[BRANCH_HASH.2..TAG.2]
            .copy_from_slice(&self.branch.as_bytes()[..BRANCH_HASH.3]);

        trytes[TAG.2..ATTACHMENT_TIMESTAMP.2]
            .copy_from_slice(&self.tag.as_bytes()[0..TAG.3]);

        trytes[ATTACHMENT_TIMESTAMP.2..ATTACHMENT_TIMESTAMP_LOWER_BOUND.2]
            .copy_from_slice(&trytes::from_num_i64_to_9(self.attachment_timestamp));

        trytes[ATTACHMENT_TIMESTAMP_LOWER_BOUND.2..ATTACHMENT_TIMESTAMP_UPPER_BOUND.2]
            .copy_from_slice(&trytes::from_num_i64_to_9(
                self.attachment_timestamp_lower_bound,
            ));

        trytes[ATTACHMENT_TIMESTAMP_UPPER_BOUND.2..NONCE.2].copy_from_slice(
            &trytes::from_num_i64_to_9(self.attachment_timestamp_upper_bound),
        );

        trytes[NONCE.2..TRANSACTION_SIZE_TRYTES]
            .copy_from_slice(&self.nonce.as_bytes()[..NONCE.3]);

        trytes
    }
}

const TRYTE_NULL_STR: &str = "9";

impl Default for Transaction {
    fn default() -> Self {
        let timestamp = time::get_unix_time_millis();

        Transaction {
            signature_fragments: TRYTE_NULL_STR.repeat(SIGNATURE_FRAGMENTS.3),
            extra_data_digest: TRYTE_NULL_STR.repeat(EXTRA_DATA_DIGEST.3),
            address: TRYTE_NULL_STR.repeat(ADDRESS.3),
            value: 0,
            issuance_timestamp: timestamp,
            timelock_lower_bound: 0,
            timelock_upper_bound: 0,
            bundle_nonce: TRYTE_NULL_STR.repeat(BUNDLE_NONCE.3),
            trunk: TRYTE_NULL_STR.repeat(TRUNK_HASH.3),
            branch: TRYTE_NULL_STR.repeat(BRANCH_HASH.3),
            tag: TRYTE_NULL_STR.repeat(TAG.3),
            attachment_timestamp: timestamp,
            attachment_timestamp_lower_bound: 0,
            attachment_timestamp_upper_bound: 0,
            nonce: TRYTE_NULL_STR.repeat(NONCE.3),
        }
    }
}

/// A transaction builder.
#[derive(Debug)]
pub struct TransactionBuilder {
    transaction: Transaction,
}

impl TransactionBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        TransactionBuilder { transaction: Transaction::default() }
    }

    /// Sets a transaction value.
    pub fn value(mut self, value: i64) -> Self {
        assert!(value.abs() <= MAX_TOKEN_SUPPLY);

        self.transaction.value = value;
        self
    }

    /// Sets a trunk hash.
    pub fn trunk(mut self, trunk: &str) -> Self {
        assert!(util::is_tryte_str(trunk));
        assert!(trunk.len() <= TRUNK_HASH.3);

        self.transaction.trunk = trunk.to_string();
        self
    }

    /// Sets a branch hash.
    pub fn branch(mut self, branch: &str) -> Self {
        assert!(util::is_tryte_str(branch));
        assert!(branch.len() <= BRANCH_HASH.3);

        self.transaction.branch = branch.to_string();
        self
    }

    /// Sets a ASCII message and stores it in the data field.
    pub fn message(mut self, message: &str) -> Self {
        assert!(message.len() <= SIGNATURE_FRAGMENTS.3);

        self.transaction.signature_fragments =
            util::pad_right(&tryte_strings::from_ascii(message), SIGNATURE_FRAGMENTS.3);
        self
    }

    /// Sets a tag.
    pub fn tag(mut self, tag: &str) -> Self {
        assert!(util::is_tryte_str(tag));
        assert!(tag.len() <= TAG.3);

        self.transaction.tag = util::pad_right(tag, TAG.3);
        self
    }

    /// Builds a transaction from the specified data.
    pub fn build(self) -> Transaction {
        self.transaction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TX_TRYTES: &str = "SEGQSWYCJHRLJYEGZLRYQAZPLVRAYIWGWJUMFFX99UZUKBQNFYAOQLOFARIKNEBKDRHJJWDJARXTNPHPAODJRSGJBVVYBVJHZALJWDCJHZRSACOVCVVAVHZVTPFTAJWVGFSVLSYXHNNXEGSMJHDBZKGFQNYJJJBAPDHFFGZ9POSOMWTDPGXI9KQRLMUVWNEQDANMXROVORJVALWVGDDJAFOOBXUKVCCIVXSSHZUCZV9XVBASLWX9NXPWGMGYCRD9ILQMKIGPBGGMKAIJKNALBLABATYFVIRBKTXTWNUZAUXRASB9EEIQHWBD9ZYUDBUPBSWXVYXQXECRCHQAYH9ZBUZBASPOIGBSGWJYFKFRITUBVMCYGCMAPTXOIWEVTUXSUOUPTUQOPMMPUTHXMOP9CW9THAZXEPMOMNEOBLUBPOAIOBEBERRZCIKHSTDWUSUPUWNJOCLNZDCEKWWAAJDPJXJEHHSYFN9MH9BGUDQ9CSZBIHRC9PSQJPGKH9ILZDWUWLEKWFKUFFFIMOQKRMKOYXEJHXLCEGCGGKHGJUHOXINSWCKRNMUNAJDCVLZGEBII9ASTYFTDYDZIZSNHIWHSQ9HODQMVNDKMKHCFDXIIGDIVJSBOOE9GRIXCD9ZUTWCUDKFTETSYSRBQABXCXZFOWQMQFXHYZWD9JZXUWHILMRNWXSGUMIIXZYCTWWHCWMSSTCNSQXQXMQPTM9MOQMIVDYNNARDCVNQEDTBKWOIOSKPKPOZHJGJJGNYWQWUWAZMBZJ9XEJMRVRYFQPJ9NOIIXEGIKMMN9DXYQUILRSCSJDIDN9DCTFGQIYWROZQIEQTKMRVLGGDGA9UVZPNRGSVTZYAPMWFUWDEUULSEEGAGITPJQ9DBEYEN9NVJPUWZTOTJHEQIXAPDOICBNNCJVDNM9YRNXMMPCOYHJDUFNCYTZGRCBZKOLHHUK9VOZWHEYQND9WUHDNGFTAS99MRCAU9QOYVUZKTIBDNAAPNEZBQPIRUFUMAWVTCXSXQQIYQPRFDUXCLJNMEIKVAINVCCZROEWEX9XVRM9IHLHQCKC9VLK9ZZWFBJUZKGJCSOPQPFVVAUDLKFJIJKMLZXFBMXLMWRSNDXRMMDLE9VBPUZB9SVLTMHA9DDDANOKIPY9ULDWAKOUDFEDHZDKMU9VMHUSFG9HRGZAZULEJJTEH9SLQDOMZTLVMBCXVNQPNKXRLBOUCCSBZRJCZIUFTFBKFVLKRBPDKLRLZSMMIQNMOZYFBGQFKUJYIJULGMVNFYJWPKPTSMYUHSUEXIPPPPPJTMDQLFFSFJFEPNUBDEDDBPGAOEJGQTHIWISLRDAABO9H9CSIAXPPJYCRFRCIH9TVBZKTCK9SPQZUYMUOKMZYOMPRHRGF9UAKZTZZG9VVVTIHMSNDREUOUOSLKUHTNFXTNSJVPVWCQXUDIMJIAMBPXUGBNDTBYPKYQYJJCDJSCTTWHOJKORLHGKRJMDCMRHSXHHMQBFJWZWHNUHZLYOAFQTRZFXDBYASYKWEVHKYDTJIAUKNCCEPSW9RITZXBOFKBAQOWHKTALQSCHARLUUGXISDMBVEUKOVXTKTEVKLGYVYHPNYWKNLCVETWIHHVTBWT9UPMTQWBZPRPRSISUBIBECVDNIZQULAGLONGVFLVZPBMHJND9CEVIXSYGFZAGGN9MQYOAKMENSEOGCUNKEJTDLEDCD9LGKYANHMZFSSDDZJKTKUJSFL9GYFDICTPJEPDSBXDQTARJQEWUVWDWSQPKIHPJONKHESSQH9FNQEO9WUCFDWPPPTIQPWCVDYTTWPLCJJVYNKE9ZEJNQBEJBMDBLNJKQDOQOHVS9VY9UPSU9KZVDFOESHNRRWBK9EZCYALAUYFGPCEWJQDXFENSNQEAUWDXJGOMCLQUQWMCPHOBZZ9SZJ9KZXSHDLPHPNYMVUJQSQETTN9SG9SIANJHWUYQXZXAJLYHCZYRGITZYQLAAYDVQVNKCDIYWAYBAFBMAYEAEAGMTJGJRSNHBHCEVIQRXEFVWJWOPU9FPDOWIFL9EWGHICRBNRITJDZNYACOGTUDBZYIYZZWAOCDBQFFNTTSTGKECWTVWZSPHX9HNRUYEAEWXENEIDLVVFMZFVPUNHMQPAIOKVIBDIHQIHFGRJOHHONPLGBSJUD9HHDTQQUZN9NVJYOAUMXMMOCNUFLZ9MXKZAGDGKVADXOVCAXEQYZGOGQKDLKIUPYXIL9PXYBQXGYDEGNXTFURSWQYLJDFKEV9VVBBQLTLHIBTFYBAJSZMDMPQHPWSFVWOJQDPHV9DYSQPIBL9LYZHQKKOVF9TFVTTXQEUWFQSLGLVTGK99VSUEDXIBIWCQHDQQSQLDHZ9999999999999999999TRINITY99999999999999999999TNXSQ9D99A99999999B99999999OGBHPUUHS9CKWSAPIMDIRNSUJ9CFPGKTUFAGQYVMFKOZSVAHIFJXWCFBZLICUWF9GNDZWCOWDUIIZ9999OXNRVXLBKJXEZMVABR9UQBVSTBDFSAJVRRNFEJRL9UFTOFPJHQMQKAJHDBIQAETS9OUVTQ9DSPAOZ9999TRINITY99999999999999999999LPZYMWQME999999999MMMMMMMMMDTIZE9999999999999999999999";
    const TX_ADDRESS: &str =
        "BAJSZMDMPQHPWSFVWOJQDPHV9DYSQPIBL9LYZHQKKOVF9TFVTTXQEUWFQSLGLVTGK99VSUEDXIBIWCQHD";

    #[test]
    fn deserialize_transaction() {
        let tx = Transaction::from_tryte_str(&TX_TRYTES);

        assert_eq!(TX_ADDRESS, tx.address);
        assert_eq!(-7_297_419_313, tx.value);
        assert_eq!(1_544_207_541_879, tx.attachment_timestamp);
    }

}
