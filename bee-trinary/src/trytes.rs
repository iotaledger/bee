//! Converter functions that convert various datatypes to Trytes.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::{
    constants::MAX_TRYTE_TRIPLET_ABS,
    constants::SIG_MSG_FRG_SIZE_TRYTES,
    constants::TRANSACTION_SIZE_TRYTES,
    constants::TRYTE_LENGTH_FOR_MAX_I64,
    luts::TRYTE_CODE_TO_ASCII_CODE,
    luts::TRYTE_CODE_TO_ASCII_CODE_NEG,
    types::Trit,
    types::Tryte,
};

macro_rules! from_bytes_conv {
    ($func_name:ident, $length:expr) => {
        /// Converts a slice of bytes to `[Tryte; $length]`.
        pub fn $func_name(bytes: &[u8]) -> [Tryte; $length] {
            #[cfg(not(feature = "no_checks"))]
            {
                assert_eq!(0, bytes.len() % 2);
                assert_eq!($length, bytes.len() / 2 * 3);
            }

            let mut trytes = [TRYTE_CODE_TO_ASCII_CODE[0]; $length];

            for i in 0..($length / 3) {
                let b0 = bytes[2 * i] as usize;
                let b1 = bytes[2 * i + 1] as usize;

                trytes[3 * i] = TRYTE_CODE_TO_ASCII_CODE[b0 / 8];
                trytes[3 * i + 1] = TRYTE_CODE_TO_ASCII_CODE[b1 / 8];
                trytes[3 * i + 2] = TRYTE_CODE_TO_ASCII_CODE[b0 % 8 + 8 * (b1 % 8)];
            }
            trytes
        }
    };
}

from_bytes_conv!(from_bytes_all, TRANSACTION_SIZE_TRYTES);
from_bytes_conv!(from_bytes_sig, SIG_MSG_FRG_SIZE_TRYTES);
from_bytes_conv!(from_bytes_81, 81);
from_bytes_conv!(from_bytes_27, 27);
from_bytes_conv!(from_bytes_9, 9);

/// Converts a slice of bytes to trytes.
pub fn from_bytes(bytes: &[u8]) -> Vec<Tryte> {
    #[cfg(not(feature = "no_checks"))]
    {
        assert_eq!(0, bytes.len() % 2);
    }

    let mut trytes = vec![TRYTE_CODE_TO_ASCII_CODE[0]; bytes.len() / 2 * 3];

    for i in 0..(trytes.len() / 3) {
        let b0 = bytes[2 * i] as usize;
        let b1 = bytes[2 * i + 1] as usize;

        trytes[3 * i] = TRYTE_CODE_TO_ASCII_CODE[b0 / 8];
        trytes[3 * i + 1] = TRYTE_CODE_TO_ASCII_CODE[b1 / 8];
        trytes[3 * i + 2] = TRYTE_CODE_TO_ASCII_CODE[b0 % 8 + 8 * (b1 % 8)];
    }
    trytes
}

macro_rules! from_trits_conv {
    ($func_name:ident, $length:expr) => {
        /// Converts fixed-sized slices of trits to trytes.
        pub fn $func_name(trits: &[Trit]) -> [Tryte; $length] {
            #[cfg(not(feature = "no_checks"))]
            {
                assert_eq!(0, trits.len() % 3);
                assert_eq!($length, trits.len() / 3);
            }

            let mut trytes = [TRYTE_CODE_TO_ASCII_CODE[0]; $length];

            for (i, t) in trytes.iter_mut().enumerate() {
                let mut index =
                    trits[i * 3] + 3 * trits[i * 3 + 1] + 9 * trits[i * 3 + 2];
                index = if index < 0 { index + 27 } else { index };
                *t = TRYTE_CODE_TO_ASCII_CODE[index as usize];
            }

            trytes
        }
    };
}

from_trits_conv!(from_trits_all, TRANSACTION_SIZE_TRYTES);
from_trits_conv!(from_trits_sig, SIG_MSG_FRG_SIZE_TRYTES);
from_trits_conv!(from_trits_243, 81);
from_trits_conv!(from_trits_81, 27);
from_trits_conv!(from_trits_27, 9);

/// Converts arbitrary slices of trits to trytes.
pub fn from_trits(trits: &[Trit]) -> Vec<Tryte> {
    #[cfg(not(feature = "no_checks"))]
    {
        assert_eq!(0, trits.len() % 3);
    }

    let mut trytes = vec![TRYTE_CODE_TO_ASCII_CODE[0]; trits.len() / 3];

    for (i, t) in trytes.iter_mut().enumerate() {
        let mut index = trits[i * 3] + 3 * trits[i * 3 + 1] + 9 * trits[i * 3 + 2];
        index = if index < 0 { index + 27 } else { index };
        *t = TRYTE_CODE_TO_ASCII_CODE[index as usize];
    }

    trytes
}

macro_rules! from_num_i64_conv {
    ($func_name:ident, $length:expr) => {
        /// Converts an `i64` number to a fixed number of trytes.
        pub fn $func_name(number: i64) -> [Tryte; $length] {
            #[cfg(not(feature = "no_checks"))]
            {
                // make sure the number can be represented by the specified
                // number of trytes
                let range_abs = (3_i64.pow(TRYTE_LENGTH_FOR_MAX_I64 as u32 * 3) - 1) / 2;
                assert!(number.abs() <= range_abs);
            }

            let lut = if number > 0 {
                &TRYTE_CODE_TO_ASCII_CODE
            } else {
                &TRYTE_CODE_TO_ASCII_CODE_NEG
            };

            let mut trytes = [TRYTE_CODE_TO_ASCII_CODE[0]; $length];

            let mut number = number.abs();
            for tryte in trytes.iter_mut().take(TRYTE_LENGTH_FOR_MAX_I64) {
                let remainder = number % 27;
                number = if remainder > 13 { number / 27 + 1 } else { number / 27 };

                *tryte = lut[remainder as usize];

                if number == 0 {
                    break;
                }
            }

            trytes
        }
    };
}

from_num_i64_conv!(from_num_i64_to_13, 13);
from_num_i64_conv!(from_num_i64_to_11, 11);
from_num_i64_conv!(from_num_i64_to_9, 9);
from_num_i64_conv!(from_num_i64_to_3, 3); //TODO: don't make this public

/// Converts an `i64` number to trytes.
pub fn from_num_i64(number: i64) -> Vec<Tryte> {
    let num_trytes =
        ((((number.abs() as f64 * 2.0) + 1.0).log(3.0)) / 3.0).ceil() as usize;

    let lut = if number > 0 {
        &TRYTE_CODE_TO_ASCII_CODE
    } else {
        &TRYTE_CODE_TO_ASCII_CODE_NEG
    };

    let mut trytes = vec![TRYTE_CODE_TO_ASCII_CODE[0]; num_trytes];

    let mut number = number.abs();

    for tryte in trytes.iter_mut() {
        let remainder = number % 27;
        number = if remainder > 13 { number / 27 + 1 } else { number / 27 };

        *tryte = lut[remainder as usize];

        if number == 0 {
            break;
        }
    }

    trytes
}

macro_rules! from_ascii_conv {
    ($func_name:ident, $length:expr) => {
        /// Converts an ASCII string to trytes.
        pub fn $func_name(ascii_str: &str) -> [Tryte; $length] {
            let mut trytes = [TRYTE_CODE_TO_ASCII_CODE[0]; $length];

            if ascii_str.is_empty() {
                return trytes;
            }

            #[cfg(not(feature = "no_checks"))]
            {
                assert!(ascii_str.is_ascii());
                assert!(ascii_str.len() == $length / 3 * 2);
            }

            let mut ascii: Vec<_> = ascii_str.chars().map(|c| c as i64).collect();

            if ascii.len() % 2 != 0 {
                ascii.push(0);
            }

            let mut tryte_index = 0;

            for i in (0..(ascii.len() - 1)).step_by(2) {
                let index = ascii[i] * 127 + ascii[i + 1] - MAX_TRYTE_TRIPLET_ABS;

                let tryte_triplet = from_num_i64_to_3(index);

                trytes[tryte_index] = tryte_triplet[0];
                trytes[tryte_index + 1] = tryte_triplet[1];
                trytes[tryte_index + 2] = tryte_triplet[2];

                tryte_index += 3;
            }

            trytes
        }
    };
}

from_ascii_conv!(from_ascii_18, 27);
from_ascii_conv!(from_ascii_6, 9);

/// Converts an ASCII string to trytes.
pub fn from_ascii(ascii_str: &str) -> Vec<Tryte> {
    if ascii_str.is_empty() {
        return vec![];
    }

    #[cfg(not(feature = "no_checks"))]
    {
        assert!(ascii_str.is_ascii());
    }

    let mut ascii: Vec<_> = ascii_str.chars().map(|c| c as i64).collect();

    if ascii.len() % 2 != 0 {
        ascii.push(0);
    }

    let mut trytes = vec![TRYTE_CODE_TO_ASCII_CODE[0]; ascii.len() / 2 * 3];

    let mut tryte_index = 0;
    for i in (0..(ascii.len() - 1)).step_by(2) {
        let index = ascii[i] * 127 + ascii[i + 1] - MAX_TRYTE_TRIPLET_ABS;

        let tryte_triplet = from_num_i64_to_3(index);

        trytes[tryte_index] = tryte_triplet[0];
        trytes[tryte_index + 1] = tryte_triplet[1];
        trytes[tryte_index + 2] = tryte_triplet[2];

        tryte_index += 3;
    }

    trytes
}

/// Converts a tryte string to trytes.
pub fn from_tryte_str(tryte_str: &str) -> Vec<Tryte> {
    tryte_str.as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANSACTION: &str = "SEGQSWYCJHRLJYEGZLRYQAZPLVRAYIWGWJUMFFX99UZUKBQNFYAOQLOFARIKNEBKDRHJJWDJARXTNPHPAODJRSGJBVVYBVJHZALJWDCJHZRSACOVCVVAVHZVTPFTAJWVGFSVLSYXHNNXEGSMJHDBZKGFQNYJJJBAPDHFFGZ9POSOMWTDPGXI9KQRLMUVWNEQDANMXROVORJVALWVGDDJAFOOBXUKVCCIVXSSHZUCZV9XVBASLWX9NXPWGMGYCRD9ILQMKIGPBGGMKAIJKNALBLABATYFVIRBKTXTWNUZAUXRASB9EEIQHWBD9ZYUDBUPBSWXVYXQXECRCHQAYH9ZBUZBASPOIGBSGWJYFKFRITUBVMCYGCMAPTXOIWEVTUXSUOUPTUQOPMMPUTHXMOP9CW9THAZXEPMOMNEOBLUBPOAIOBEBERRZCIKHSTDWUSUPUWNJOCLNZDCEKWWAAJDPJXJEHHSYFN9MH9BGUDQ9CSZBIHRC9PSQJPGKH9ILZDWUWLEKWFKUFFFIMOQKRMKOYXEJHXLCEGCGGKHGJUHOXINSWCKRNMUNAJDCVLZGEBII9ASTYFTDYDZIZSNHIWHSQ9HODQMVNDKMKHCFDXIIGDIVJSBOOE9GRIXCD9ZUTWCUDKFTETSYSRBQABXCXZFOWQMQFXHYZWD9JZXUWHILMRNWXSGUMIIXZYCTWWHCWMSSTCNSQXQXMQPTM9MOQMIVDYNNARDCVNQEDTBKWOIOSKPKPOZHJGJJGNYWQWUWAZMBZJ9XEJMRVRYFQPJ9NOIIXEGIKMMN9DXYQUILRSCSJDIDN9DCTFGQIYWROZQIEQTKMRVLGGDGA9UVZPNRGSVTZYAPMWFUWDEUULSEEGAGITPJQ9DBEYEN9NVJPUWZTOTJHEQIXAPDOICBNNCJVDNM9YRNXMMPCOYHJDUFNCYTZGRCBZKOLHHUK9VOZWHEYQND9WUHDNGFTAS99MRCAU9QOYVUZKTIBDNAAPNEZBQPIRUFUMAWVTCXSXQQIYQPRFDUXCLJNMEIKVAINVCCZROEWEX9XVRM9IHLHQCKC9VLK9ZZWFBJUZKGJCSOPQPFVVAUDLKFJIJKMLZXFBMXLMWRSNDXRMMDLE9VBPUZB9SVLTMHA9DDDANOKIPY9ULDWAKOUDFEDHZDKMU9VMHUSFG9HRGZAZULEJJTEH9SLQDOMZTLVMBCXVNQPNKXRLBOUCCSBZRJCZIUFTFBKFVLKRBPDKLRLZSMMIQNMOZYFBGQFKUJYIJULGMVNFYJWPKPTSMYUHSUEXIPPPPPJTMDQLFFSFJFEPNUBDEDDBPGAOEJGQTHIWISLRDAABO9H9CSIAXPPJYCRFRCIH9TVBZKTCK9SPQZUYMUOKMZYOMPRHRGF9UAKZTZZG9VVVTIHMSNDREUOUOSLKUHTNFXTNSJVPVWCQXUDIMJIAMBPXUGBNDTBYPKYQYJJCDJSCTTWHOJKORLHGKRJMDCMRHSXHHMQBFJWZWHNUHZLYOAFQTRZFXDBYASYKWEVHKYDTJIAUKNCCEPSW9RITZXBOFKBAQOWHKTALQSCHARLUUGXISDMBVEUKOVXTKTEVKLGYVYHPNYWKNLCVETWIHHVTBWT9UPMTQWBZPRPRSISUBIBECVDNIZQULAGLONGVFLVZPBMHJND9CEVIXSYGFZAGGN9MQYOAKMENSEOGCUNKEJTDLEDCD9LGKYANHMZFSSDDZJKTKUJSFL9GYFDICTPJEPDSBXDQTARJQEWUVWDWSQPKIHPJONKHESSQH9FNQEO9WUCFDWPPPTIQPWCVDYTTWPLCJJVYNKE9ZEJNQBEJBMDBLNJKQDOQOHVS9VY9UPSU9KZVDFOESHNRRWBK9EZCYALAUYFGPCEWJQDXFENSNQEAUWDXJGOMCLQUQWMCPHOBZZ9SZJ9KZXSHDLPHPNYMVUJQSQETTN9SG9SIANJHWUYQXZXAJLYHCZYRGITZYQLAAYDVQVNKCDIYWAYBAFBMAYEAEAGMTJGJRSNHBHCEVIQRXEFVWJWOPU9FPDOWIFL9EWGHICRBNRITJDZNYACOGTUDBZYIYZZWAOCDBQFFNTTSTGKECWTVWZSPHX9HNRUYEAEWXENEIDLVVFMZFVPUNHMQPAIOKVIBDIHQIHFGRJOHHONPLGBSJUD9HHDTQQUZN9NVJYOAUMXMMOCNUFLZ9MXKZAGDGKVADXOVCAXEQYZGOGQKDLKIUPYXIL9PXYBQXGYDEGNXTFURSWQYLJDFKEV9VVBBQLTLHIBTFYBAJSZMDMPQHPWSFVWOJQDPHV9DYSQPIBL9LYZHQKKOVF9TFVTTXQEUWFQSLGLVTGK99VSUEDXIBIWCQHDQQSQLDHZ9999999999999999999TRINITY99999999999999999999TNXSQ9D99A99999999B99999999OGBHPUUHS9CKWSAPIMDIRNSUJ9CFPGKTUFAGQYVMFKOZSVAHIFJXWCFBZLICUWF9GNDZWCOWDUIIZ9999OXNRVXLBKJXEZMVABR9UQBVSTBDFSAJVRRNFEJRL9UFTOFPJHQMQKAJHDBIQAETS9OUVTQ9DSPAOZ9999TRINITY99999999999999999999LPZYMWQME999999999MMMMMMMMMDTIZE9999999999999999999999";

    #[test]
    fn test_from_bytes() {
        assert_eq!(
            &[57, 57, 57, 57],
            &crate::bytes::from_trytes(&from_bytes(&[57, 57, 57, 57]))[..]
        );

        let bytes = crate::bytes::from_trytes(&from_tryte_str(TRANSACTION));

        assert_eq!(&bytes[..], &crate::bytes::from_trytes(&from_bytes(&bytes))[..]);
    }

    #[test]
    fn test_from_trits_all() {
        let all_trits = &crate::trits::from_tryte_str(TRANSACTION)[..];

        assert_eq!(
            &all_trits[..],
            &crate::trits::from_trytes(&from_trits_all(&all_trits))[..]
        );

        let sig_trits = &crate::trits::from_tryte_str(TRANSACTION)[0..6561];

        assert_eq!(
            &sig_trits[..],
            &crate::trits::from_trytes(&from_trits_sig(&sig_trits))[..]
        );
    }

    #[test]
    fn test_from_trits_27() {
        let trits = [
            1, 1, 0, -1, -1, 0, -1, 1, 0, 1, -1, 0, 1, 1, 1, 1, -1, -1, 1, -1, 0, 1, -1,
            0, 1, -1, 0,
        ];

        assert_eq!(&trits, &crate::trits::from_trytes(&from_trits_27(&trits))[..]);
    }

    #[test]
    fn test_from_trits() {
        assert_eq!(&[1, 0, 0], &crate::trits::from_trytes(&from_trits(&[1, 0, 0]))[..]);
        assert_eq!(
            &[1, -1, 0, -1, 1, 0],
            &crate::trits::from_trytes(&from_trits(&[1, -1, 0, -1, 1, 0]))[..]
        );
    }

    #[test]
    fn test_from_num_i64() {
        assert_eq!(729, crate::numbers::from_trytes_max13(&from_num_i64(729)[..]));

        assert_eq!(0, crate::numbers::from_trytes_max13(&from_num_i64(0)[..]));

        assert_eq!(
            1234567890,
            crate::numbers::from_trytes_max13(&from_num_i64(1234567890)[..])
        );

        assert_eq!(
            core::i64::MAX / 8,
            crate::numbers::from_trytes_max13(&from_num_i64(core::i64::MAX / 8)[..])
        );
    }

    #[test]
    fn test_from_num_i64_to_13() {
        assert_eq!(729, crate::numbers::from_trytes_max13(&from_num_i64_to_13(729)[..]));
    }

    #[test]
    fn test_from_num_i64_to_11() {
        assert_eq!(729, crate::numbers::from_trytes_max11(&from_num_i64_to_11(729)[..]));
    }

    #[test]
    fn test_from_ascii_6() {
        assert_eq!(
            "Hello!",
            crate::ascii_strings::from_trytes(&from_ascii_6("Hello!")[..])
        );
    }

    #[test]
    fn test_from_ascii_18() {
        assert_eq!(
            "Hello, Multiverse!",
            crate::ascii_strings::from_trytes(&from_ascii_18("Hello, Multiverse!")[..])
        );
    }

    #[test]
    fn test_from_ascii() {
        assert_eq!(
            "Hello, World!",
            crate::ascii_strings::from_trytes(&from_ascii("Hello, World!")[..])
        );
        assert_eq!("Hel", crate::ascii_strings::from_trytes(&from_ascii("Hel")[..]));
    }
}
