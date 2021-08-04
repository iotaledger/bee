#![allow(unused_imports)]

use bee_packable::Packable;

use core::convert::Infallible;

#[derive(Packable)]
#[packable(pack_error = Impossible, with = Impossible::from)]
#[packable(unpack_error = Impossible, with = Impossible::from)]
pub struct Point {
    x: i32,
    y: i32,
}

pub enum Impossible {}

impl From<Infallible> for Impossible {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

fn main() {}
