extern crate failure;

pub use self::ftroika::*;
pub use self::troika::*;

mod constants;
pub mod ftroika;
pub mod troika;

use core::result;

pub type Result<T> = result::Result<T, failure::Error>;
