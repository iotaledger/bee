//#![no_std]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate failure;

pub mod trytes;

pub use self::trytes::*;

type Result<T> = ::core::result::Result<T, failure::Error>;

#[test]
fn decode() {
    let t = Trytes::try_from("VB").unwrap();
    let x = t.decode();
    println!("{:?}", x);
}
