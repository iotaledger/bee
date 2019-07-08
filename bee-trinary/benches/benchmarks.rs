#[macro_use]
extern crate criterion;
extern crate bee_conv;

use criterion::{black_box, Criterion};
use rand::seq::SliceRandom;

const TRANSACTION: &str = "SEGQSWYCJHRLJYEGZLRYQAZPLVRAYIWGWJUMFFX99UZUKBQNFYAOQLOFARIKNEBKDRHJJWDJARXTNPHPAODJRSGJBVVYBVJHZALJWDCJHZRSACOVCVVAVHZVTPFTAJWVGFSVLSYXHNNXEGSMJHDBZKGFQNYJJJBAPDHFFGZ9POSOMWTDPGXI9KQRLMUVWNEQDANMXROVORJVALWVGDDJAFOOBXUKVCCIVXSSHZUCZV9XVBASLWX9NXPWGMGYCRD9ILQMKIGPBGGMKAIJKNALBLABATYFVIRBKTXTWNUZAUXRASB9EEIQHWBD9ZYUDBUPBSWXVYXQXECRCHQAYH9ZBUZBASPOIGBSGWJYFKFRITUBVMCYGCMAPTXOIWEVTUXSUOUPTUQOPMMPUTHXMOP9CW9THAZXEPMOMNEOBLUBPOAIOBEBERRZCIKHSTDWUSUPUWNJOCLNZDCEKWWAAJDPJXJEHHSYFN9MH9BGUDQ9CSZBIHRC9PSQJPGKH9ILZDWUWLEKWFKUFFFIMOQKRMKOYXEJHXLCEGCGGKHGJUHOXINSWCKRNMUNAJDCVLZGEBII9ASTYFTDYDZIZSNHIWHSQ9HODQMVNDKMKHCFDXIIGDIVJSBOOE9GRIXCD9ZUTWCUDKFTETSYSRBQABXCXZFOWQMQFXHYZWD9JZXUWHILMRNWXSGUMIIXZYCTWWHCWMSSTCNSQXQXMQPTM9MOQMIVDYNNARDCVNQEDTBKWOIOSKPKPOZHJGJJGNYWQWUWAZMBZJ9XEJMRVRYFQPJ9NOIIXEGIKMMN9DXYQUILRSCSJDIDN9DCTFGQIYWROZQIEQTKMRVLGGDGA9UVZPNRGSVTZYAPMWFUWDEUULSEEGAGITPJQ9DBEYEN9NVJPUWZTOTJHEQIXAPDOICBNNCJVDNM9YRNXMMPCOYHJDUFNCYTZGRCBZKOLHHUK9VOZWHEYQND9WUHDNGFTAS99MRCAU9QOYVUZKTIBDNAAPNEZBQPIRUFUMAWVTCXSXQQIYQPRFDUXCLJNMEIKVAINVCCZROEWEX9XVRM9IHLHQCKC9VLK9ZZWFBJUZKGJCSOPQPFVVAUDLKFJIJKMLZXFBMXLMWRSNDXRMMDLE9VBPUZB9SVLTMHA9DDDANOKIPY9ULDWAKOUDFEDHZDKMU9VMHUSFG9HRGZAZULEJJTEH9SLQDOMZTLVMBCXVNQPNKXRLBOUCCSBZRJCZIUFTFBKFVLKRBPDKLRLZSMMIQNMOZYFBGQFKUJYIJULGMVNFYJWPKPTSMYUHSUEXIPPPPPJTMDQLFFSFJFEPNUBDEDDBPGAOEJGQTHIWISLRDAABO9H9CSIAXPPJYCRFRCIH9TVBZKTCK9SPQZUYMUOKMZYOMPRHRGF9UAKZTZZG9VVVTIHMSNDREUOUOSLKUHTNFXTNSJVPVWCQXUDIMJIAMBPXUGBNDTBYPKYQYJJCDJSCTTWHOJKORLHGKRJMDCMRHSXHHMQBFJWZWHNUHZLYOAFQTRZFXDBYASYKWEVHKYDTJIAUKNCCEPSW9RITZXBOFKBAQOWHKTALQSCHARLUUGXISDMBVEUKOVXTKTEVKLGYVYHPNYWKNLCVETWIHHVTBWT9UPMTQWBZPRPRSISUBIBECVDNIZQULAGLONGVFLVZPBMHJND9CEVIXSYGFZAGGN9MQYOAKMENSEOGCUNKEJTDLEDCD9LGKYANHMZFSSDDZJKTKUJSFL9GYFDICTPJEPDSBXDQTARJQEWUVWDWSQPKIHPJONKHESSQH9FNQEO9WUCFDWPPPTIQPWCVDYTTWPLCJJVYNKE9ZEJNQBEJBMDBLNJKQDOQOHVS9VY9UPSU9KZVDFOESHNRRWBK9EZCYALAUYFGPCEWJQDXFENSNQEAUWDXJGOMCLQUQWMCPHOBZZ9SZJ9KZXSHDLPHPNYMVUJQSQETTN9SG9SIANJHWUYQXZXAJLYHCZYRGITZYQLAAYDVQVNKCDIYWAYBAFBMAYEAEAGMTJGJRSNHBHCEVIQRXEFVWJWOPU9FPDOWIFL9EWGHICRBNRITJDZNYACOGTUDBZYIYZZWAOCDBQFFNTTSTGKECWTVWZSPHX9HNRUYEAEWXENEIDLVVFMZFVPUNHMQPAIOKVIBDIHQIHFGRJOHHONPLGBSJUD9HHDTQQUZN9NVJYOAUMXMMOCNUFLZ9MXKZAGDGKVADXOVCAXEQYZGOGQKDLKIUPYXIL9PXYBQXGYDEGNXTFURSWQYLJDFKEV9VVBBQLTLHIBTFYBAJSZMDMPQHPWSFVWOJQDPHV9DYSQPIBL9LYZHQKKOVF9TFVTTXQEUWFQSLGLVTGK99VSUEDXIBIWCQHDQQSQLDHZ9999999999999999999TRINITY99999999999999999999TNXSQ9D99A99999999B99999999OGBHPUUHS9CKWSAPIMDIRNSUJ9CFPGKTUFAGQYVMFKOZSVAHIFJXWCFBZLICUWF9GNDZWCOWDUIIZ9999OXNRVXLBKJXEZMVABR9UQBVSTBDFSAJVRRNFEJRL9UFTOFPJHQMQKAJHDBIQAETS9OUVTQ9DSPAOZ9999TRINITY99999999999999999999LPZYMWQME999999999MMMMMMMMMDTIZE9999999999999999999999";
const TRYTE_ALPHABET: &[u8] = b"9ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const SIG_MSG_FRAG_SIZE_TRYTES: usize = 2187;

// Helper function to get some random trytes.
fn get_random_tryte_str() -> String {
    let mut rng = rand::thread_rng();
    (0..SIG_MSG_FRAG_SIZE_TRYTES)
        .map(|_| *TRYTE_ALPHABET.choose(&mut rng).unwrap() as char)
        .collect::<String>()
}

// Helper function to get some tryte encoded ASCII text.
fn get_repeated_tryte_str(num: usize) -> String {
    "YEZNMEQWF".repeat(num)
}

// Helper function to get some transaction trytes.
fn get_transaction_trytes() -> &'static [u8] {
    TRANSACTION.as_bytes()
}

// Helper function to get some transaction trits.
fn get_transaction_trits() -> Vec<i8> {
    bee_conv::trits::from_trytes(&TRANSACTION.as_bytes())
}

fn ascii_strings_from_tryte_str_benchmarks(c: &mut Criterion) {
    // 2187 trytes
    c.bench_function("ascii_string::from_trytes (2187)", move |b| {
        b.iter(|| bee_conv::ascii_strings::from_tryte_str(&get_repeated_tryte_str(243)))
    });
    // 729 trytes
    c.bench_function("ascii_string::from_trytes (729)", move |b| {
        b.iter(|| bee_conv::ascii_strings::from_tryte_str(&get_repeated_tryte_str(81)))
    });
    // 243 trytes
    c.bench_function("ascii_string::from_trytes (243)", move |b| {
        b.iter(|| bee_conv::ascii_strings::from_tryte_str(&get_repeated_tryte_str(27)))
    });
}

fn bytes_from_trytes_benchmarks(c: &mut Criterion) {
    c.bench_function("bytes::from_trytes_all", move |b| {
        b.iter(|| bee_conv::bytes::from_trytes_all(&get_transaction_trytes()))
    });
    c.bench_function("bytes::from_trytes_sig", move |b| {
        b.iter(|| bee_conv::bytes::from_trytes_sig(&get_transaction_trytes()[0..2187]))
    });
    c.bench_function("bytes::from_trytes_81", move |b| {
        b.iter(|| bee_conv::bytes::from_trytes_81(&get_transaction_trytes()[2187..2268]))
    });
    c.bench_function("bytes::from_trytes_27", move |b| {
        b.iter(|| bee_conv::bytes::from_trytes_27(&get_transaction_trytes()[2349..2376]))
    });
    c.bench_function("bytes::from_trytes_9", move |b| {
        b.iter(|| bee_conv::bytes::from_trytes_9(&get_transaction_trytes()[2376..2385]))
    });
    c.bench_function("bytes::from_trytes", move |b| {
        b.iter(|| bee_conv::bytes::from_trytes(&get_transaction_trytes()))
    });
}

fn bytes_from_trits_benchmarks(c: &mut Criterion) {
    c.bench_function("bytes::from_trits_all", move |b| {
        b.iter(|| bee_conv::bytes::from_trits_all(&get_transaction_trits()))
    });
    c.bench_function("bytes::from_trits_sig", move |b| {
        b.iter(|| bee_conv::bytes::from_trits_sig(&get_transaction_trits()[0..6561]))
    });
    c.bench_function("bytes::from_trits_243", move |b| {
        b.iter(|| bee_conv::bytes::from_trits_243(&get_transaction_trits()[6561..6804]))
    });
    c.bench_function("bytes::from_trits_81", move |b| {
        b.iter(|| bee_conv::bytes::from_trits_81(&get_transaction_trits()[7047..7128]))
    });
    c.bench_function("bytes::from_trits_27", move |b| {
        b.iter(|| bee_conv::bytes::from_trits_27(&get_transaction_trits()[7857..7884]))
    });
    c.bench_function("bytes::from_trits", move |b| {
        b.iter(|| bee_conv::bytes::from_trits(&get_transaction_trits()))
    });
}

// Trytes -> Number (i64, s129)
fn numbers_from_trytes_benchmarks(c: &mut Criterion) {
    c.bench_function("numbers::from_trytes_max11", move |b| {
        b.iter(|| {
            bee_conv::numbers::from_trytes_max11(&get_transaction_trytes()[2349..2360])
        })
    });
    c.bench_function("numbers::from_trytes_max13", move |b| {
        b.iter(|| {
            bee_conv::numbers::from_trytes_max13(&get_transaction_trytes()[2349..2360])
        })
    });
    c.bench_function("numbers::from_trytes_max27", move |b| {
        b.iter(|| {
            bee_conv::numbers::from_trytes_max27(&get_transaction_trytes()[2349..2360])
        })
    });
}

// Trits -> Number (i64)
fn numbers_from_trits_benchmarks(c: &mut Criterion) {
    c.bench_function("numbera::from_trits", move |b| {
        b.iter(|| bee_conv::numbers::from_trits(&get_transaction_trits()[..]))
    });
}

fn add_numbers_benchmarks(c: &mut Criterion) {
    //c.bench_function("add_i64s", move |b| b.iter(|| add_i64s()));
    c.bench_function("add_u128s", move |b| b.iter(|| add_u128s()));
    c.bench_function("add_i129s", move |b| b.iter(|| add_i129s()));
}

use bee_conv::types::{Sign, I129};

const I129_MAX: u128 = 1_000_000;

fn add_u128s() {
    let mut v = 0_u128;
    for i in 1..=I129_MAX {
        v += i;
    }
    assert_eq!(500_000_500_000, v);
}

fn add_i129s() {
    let mut v = I129::from(0);
    for i in 1..=I129_MAX {
        v += I129(Sign::Pos, i);
    }
    assert_eq!(I129(Sign::Pos, 500_000_500_000), v);
}

/*
/// The sign for the `I129` signed 129 bit integer.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Sign {
    /// Positive sign.
    Pos,
    /// Negative sign.
    Neg,
}

/// A signed 129 bit integer to represent any positive or negative integer that can be encoded with
/// up to 27 (balanced trinary) trytes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct I129(pub Sign, pub u128);

impl I129 {
    /// Creates a I129 representing the tuple (+, 0).
    pub fn zero() -> Self {
        I129(Sign::Pos, 0)
    }
}

impl From<u128> for I129 {
    fn from(n: u128) -> Self {
        Self(Sign::Pos, n)
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl std::ops::AddAssign for I129 {
    fn add_assign(&mut self, other: Self) {
        match (&self.0, other.0) {
            (Sign::Pos, Sign::Neg) => {
                if other.1 > self.1 {
                    self.0 = Sign::Neg;
                    self.1 = other.1 - self.1;
                } else {
                    self.1 -= other.1;
                }
            }
            (Sign::Neg, Sign::Pos) => {
                if other.1 >= self.1 {
                    self.0 = Sign::Pos;
                    self.1 = other.1 - self.1;
                } else {
                    self.1 -= other.1;
                }
            }
            (_, _) => self.1 += other.1,
        }
    }
}
*/

// --------------------------------------------------------------

criterion_group!(
    benches,
    //ascii_strings_from_tryte_str_benchmarks,
    //bytes_from_trytes_benchmarks,
    //bytes_from_trits_benchmarks,
    //numbers_from_trytes_benchmarks,
    add_numbers_benchmarks,
);
criterion_main!(benches);
