//! Converter functions that convert various datatypes to Bytes.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::{
    constants::SIG_MSG_FRG_SIZE_BYTES,
    constants::TRANSACTION_SIZE_BYTES,
    luts::TRYTE_CODE_TO_ASCII_CODE,
    types::Byte,
    types::Trit,
    types::Tryte,
};

const TRYTE_9: u8 = TRYTE_CODE_TO_ASCII_CODE[0];
const TRYTE_A: u8 = TRYTE_CODE_TO_ASCII_CODE[1];

macro_rules! from_trytes_conv {
    ($func_name:ident, $length:expr) => {
        /// Converts fixed-sized slices of trytes to bytes.
        pub fn $func_name(trytes: &[Tryte]) -> [Byte; $length] {
            #[cfg(not(feature = "no_checks"))]
            {
                assert_eq!(0, trytes.len() % 3);
                assert_eq!($length, trytes.len() / 3 * 2);
            }
            let mut bytes = [0_u8; $length];

            for i in 0..trytes.len() / 3 {
                let t0 = trytes[3 * i];
                let t1 = trytes[3 * i + 1];
                let t2 = trytes[3 * i + 2];

                let i0 = if t0 == TRYTE_9 { 0 } else { t0 - TRYTE_A + 1 };
                let i1 = if t1 == TRYTE_9 { 0 } else { t1 - TRYTE_A + 1 };
                let i2 = if t2 == TRYTE_9 { 0 } else { t2 - TRYTE_A + 1 };

                bytes[2 * i] = i0 * 8 + i2 % 8;
                bytes[2 * i + 1] = i1 * 8 + i2 / 8;
            }
            bytes
        }
    };
}

from_trytes_conv!(from_trytes_all, TRANSACTION_SIZE_BYTES);
from_trytes_conv!(from_trytes_sig, SIG_MSG_FRG_SIZE_BYTES);
from_trytes_conv!(from_trytes_81, 54);
from_trytes_conv!(from_trytes_27, 18);
from_trytes_conv!(from_trytes_9, 6);

/// Converts arbitrary slices of trytes to bytes.
pub fn from_trytes(trytes: &[Tryte]) -> Vec<Byte> {
    #[cfg(not(feature = "no_checks"))]
    {
        assert_eq!(0, trytes.len() % 3);
    }

    let mut bytes = vec![0_u8; trytes.len() / 3 * 2];

    for i in 0..trytes.len() / 3 {
        let t0 = trytes[3 * i];
        let t1 = trytes[3 * i + 1];
        let t2 = trytes[3 * i + 2];

        let i0 = if t0 == TRYTE_9 { 0 } else { t0 - TRYTE_A + 1 };
        let i1 = if t1 == TRYTE_9 { 0 } else { t1 - TRYTE_A + 1 };
        let i2 = if t2 == TRYTE_9 { 0 } else { t2 - TRYTE_A + 1 };

        bytes[2 * i] = i0 * 8 + i2 % 8;
        bytes[2 * i + 1] = i1 * 8 + i2 / 8;
    }
    bytes
}

macro_rules! from_trits_conv {
    ($func_name:ident, $length:expr) => {
        /// Converts fixed-sized slices of trits to bytes.
        pub fn $func_name(trits: &[Trit]) -> [Byte; $length] {
            #[cfg(not(feature = "no_checks"))]
            {
                assert_eq!(0, trits.len() % 9);
                assert_eq!($length, trits.len() / 9 * 2);
            }
            let mut bytes = [0_u8; $length];

            for i in 0..(trits.len() / 9) {
                let i0 = trits[9 * i] + 3 * trits[9 * i + 1] + 9 * trits[9 * i + 2];
                let i1 = trits[9 * i + 3] + 3 * trits[9 * i + 4] + 9 * trits[9 * i + 5];
                let i2 = trits[9 * i + 6] + 3 * trits[9 * i + 7] + 9 * trits[9 * i + 8];

                let j0 = if i0 < 0 { i0 + 27 } else { i0 } as u8;
                let j1 = if i1 < 0 { i1 + 27 } else { i1 } as u8;
                let j2 = if i2 < 0 { i2 + 27 } else { i2 } as u8;

                bytes[2 * i] = j0 * 8 + j2 % 8;
                bytes[2 * i + 1] = j1 * 8 + j2 / 8;
            }
            bytes
        }
    };
}

from_trits_conv!(from_trits_all, TRANSACTION_SIZE_BYTES);
from_trits_conv!(from_trits_sig, SIG_MSG_FRG_SIZE_BYTES);
from_trits_conv!(from_trits_243, 54);
from_trits_conv!(from_trits_81, 18);
from_trits_conv!(from_trits_27, 6);

/// Converts fixed-sized slices of trits to bytes.
pub fn from_trits(trits: &[Trit]) -> Vec<Byte> {
    #[cfg(not(feature = "no_checks"))]
    {
        assert_eq!(0, trits.len() % 9);
    }

    let mut bytes = vec![0_u8; trits.len() / 9 * 2];

    for i in 0..(trits.len() / 9) {
        let i0 = trits[9 * i] + 3 * trits[9 * i + 1] + 9 * trits[9 * i + 2];
        let i1 = trits[9 * i + 3] + 3 * trits[9 * i + 4] + 9 * trits[9 * i + 5];
        let i2 = trits[9 * i + 6] + 3 * trits[9 * i + 7] + 9 * trits[9 * i + 8];

        let j0 = if i0 < 0 { i0 + 27 } else { i0 } as u8;
        let j1 = if i1 < 0 { i1 + 27 } else { i1 } as u8;
        let j2 = if i2 < 0 { i2 + 27 } else { i2 } as u8;

        bytes[2 * i] = j0 * 8 + j2 % 8;
        bytes[2 * i + 1] = j1 * 8 + j2 / 8;
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANSACTION: &str = "SEGQSWYCJHRLJYEGZLRYQAZPLVRAYIWGWJUMFFX99UZUKBQNFYAOQLOFARIKNEBKDRHJJWDJARXTNPHPAODJRSGJBVVYBVJHZALJWDCJHZRSACOVCVVAVHZVTPFTAJWVGFSVLSYXHNNXEGSMJHDBZKGFQNYJJJBAPDHFFGZ9POSOMWTDPGXI9KQRLMUVWNEQDANMXROVORJVALWVGDDJAFOOBXUKVCCIVXSSHZUCZV9XVBASLWX9NXPWGMGYCRD9ILQMKIGPBGGMKAIJKNALBLABATYFVIRBKTXTWNUZAUXRASB9EEIQHWBD9ZYUDBUPBSWXVYXQXECRCHQAYH9ZBUZBASPOIGBSGWJYFKFRITUBVMCYGCMAPTXOIWEVTUXSUOUPTUQOPMMPUTHXMOP9CW9THAZXEPMOMNEOBLUBPOAIOBEBERRZCIKHSTDWUSUPUWNJOCLNZDCEKWWAAJDPJXJEHHSYFN9MH9BGUDQ9CSZBIHRC9PSQJPGKH9ILZDWUWLEKWFKUFFFIMOQKRMKOYXEJHXLCEGCGGKHGJUHOXINSWCKRNMUNAJDCVLZGEBII9ASTYFTDYDZIZSNHIWHSQ9HODQMVNDKMKHCFDXIIGDIVJSBOOE9GRIXCD9ZUTWCUDKFTETSYSRBQABXCXZFOWQMQFXHYZWD9JZXUWHILMRNWXSGUMIIXZYCTWWHCWMSSTCNSQXQXMQPTM9MOQMIVDYNNARDCVNQEDTBKWOIOSKPKPOZHJGJJGNYWQWUWAZMBZJ9XEJMRVRYFQPJ9NOIIXEGIKMMN9DXYQUILRSCSJDIDN9DCTFGQIYWROZQIEQTKMRVLGGDGA9UVZPNRGSVTZYAPMWFUWDEUULSEEGAGITPJQ9DBEYEN9NVJPUWZTOTJHEQIXAPDOICBNNCJVDNM9YRNXMMPCOYHJDUFNCYTZGRCBZKOLHHUK9VOZWHEYQND9WUHDNGFTAS99MRCAU9QOYVUZKTIBDNAAPNEZBQPIRUFUMAWVTCXSXQQIYQPRFDUXCLJNMEIKVAINVCCZROEWEX9XVRM9IHLHQCKC9VLK9ZZWFBJUZKGJCSOPQPFVVAUDLKFJIJKMLZXFBMXLMWRSNDXRMMDLE9VBPUZB9SVLTMHA9DDDANOKIPY9ULDWAKOUDFEDHZDKMU9VMHUSFG9HRGZAZULEJJTEH9SLQDOMZTLVMBCXVNQPNKXRLBOUCCSBZRJCZIUFTFBKFVLKRBPDKLRLZSMMIQNMOZYFBGQFKUJYIJULGMVNFYJWPKPTSMYUHSUEXIPPPPPJTMDQLFFSFJFEPNUBDEDDBPGAOEJGQTHIWISLRDAABO9H9CSIAXPPJYCRFRCIH9TVBZKTCK9SPQZUYMUOKMZYOMPRHRGF9UAKZTZZG9VVVTIHMSNDREUOUOSLKUHTNFXTNSJVPVWCQXUDIMJIAMBPXUGBNDTBYPKYQYJJCDJSCTTWHOJKORLHGKRJMDCMRHSXHHMQBFJWZWHNUHZLYOAFQTRZFXDBYASYKWEVHKYDTJIAUKNCCEPSW9RITZXBOFKBAQOWHKTALQSCHARLUUGXISDMBVEUKOVXTKTEVKLGYVYHPNYWKNLCVETWIHHVTBWT9UPMTQWBZPRPRSISUBIBECVDNIZQULAGLONGVFLVZPBMHJND9CEVIXSYGFZAGGN9MQYOAKMENSEOGCUNKEJTDLEDCD9LGKYANHMZFSSDDZJKTKUJSFL9GYFDICTPJEPDSBXDQTARJQEWUVWDWSQPKIHPJONKHESSQH9FNQEO9WUCFDWPPPTIQPWCVDYTTWPLCJJVYNKE9ZEJNQBEJBMDBLNJKQDOQOHVS9VY9UPSU9KZVDFOESHNRRWBK9EZCYALAUYFGPCEWJQDXFENSNQEAUWDXJGOMCLQUQWMCPHOBZZ9SZJ9KZXSHDLPHPNYMVUJQSQETTN9SG9SIANJHWUYQXZXAJLYHCZYRGITZYQLAAYDVQVNKCDIYWAYBAFBMAYEAEAGMTJGJRSNHBHCEVIQRXEFVWJWOPU9FPDOWIFL9EWGHICRBNRITJDZNYACOGTUDBZYIYZZWAOCDBQFFNTTSTGKECWTVWZSPHX9HNRUYEAEWXENEIDLVVFMZFVPUNHMQPAIOKVIBDIHQIHFGRJOHHONPLGBSJUD9HHDTQQUZN9NVJYOAUMXMMOCNUFLZ9MXKZAGDGKVADXOVCAXEQYZGOGQKDLKIUPYXIL9PXYBQXGYDEGNXTFURSWQYLJDFKEV9VVBBQLTLHIBTFYBAJSZMDMPQHPWSFVWOJQDPHV9DYSQPIBL9LYZHQKKOVF9TFVTTXQEUWFQSLGLVTGK99VSUEDXIBIWCQHDQQSQLDHZ9999999999999999999TRINITY99999999999999999999TNXSQ9D99A99999999B99999999OGBHPUUHS9CKWSAPIMDIRNSUJ9CFPGKTUFAGQYVMFKOZSVAHIFJXWCFBZLICUWF9GNDZWCOWDUIIZ9999OXNRVXLBKJXEZMVABR9UQBVSTBDFSAJVRRNFEJRL9UFTOFPJHQMQKAJHDBIQAETS9OUVTQ9DSPAOZ9999TRINITY99999999999999999999LPZYMWQME999999999MMMMMMMMMDTIZE9999999999999999999999";

    #[test]
    fn test_from_trytes_all() {
        let tx = TRANSACTION.as_bytes();

        assert_eq!(
            tx,
            &crate::trytes::from_bytes_all(&from_trytes_all(&tx[..])).to_vec()[..]
        );
    }

    #[test]
    fn test_from_trytes_sig() {
        let sig = &TRANSACTION.as_bytes()[0..2187];

        assert_eq!(
            sig,
            &crate::trytes::from_bytes_sig(&from_trytes_sig(sig)).to_vec()[..]
        );
    }

    #[test]
    fn test_from_trytes() {
        let trytes = &TRANSACTION.as_bytes()[13..574];

        assert_eq!(
            trytes,
            &crate::trytes::from_bytes(&from_trytes(trytes)).to_vec()[..]
        );
    }

    #[test]
    fn test_from_trits_all() {
        let tx = TRANSACTION.as_bytes();
        let trits = crate::trits::from_trytes(&tx);

        assert_eq!(
            tx,
            &crate::trytes::from_bytes_all(&from_trits_all(&trits[..])).to_vec()[..]
        );
    }

    #[test]
    fn test_from_trits_sig() {
        let sig = &TRANSACTION.as_bytes()[0..2187];
        let trits = crate::trits::from_trytes(sig);

        assert_eq!(
            sig,
            &crate::trytes::from_bytes_sig(&from_trits_sig(&trits)).to_vec()[..]
        );
    }

    #[test]
    fn test_from_trits() {
        let trytes = &TRANSACTION.as_bytes()[13..574];
        let trits = crate::trits::from_trytes(trytes);

        assert_eq!(
            trytes,
            &crate::trytes::from_bytes(&from_trits(&trits)).to_vec()[..]
        );
    }
}
