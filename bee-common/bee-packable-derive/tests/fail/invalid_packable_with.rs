#![allow(unused_imports)]

use bee_packable::Packable;

use core::convert::Infallible;

#[derive(Packable)]
#[packable(pack_error = Impossible, with = Impossible::new)]
#[packable(unpack_error = Impossible, with = Impossible::new)]
pub struct Point {
    x: i32,
    y: i32,
}

pub enum Impossible {}

impl Impossible {
    fn new() -> Self {
        unreachable!()
    }
}

fn main() {}
