// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

mod hash;
mod message;
pub mod payload;

pub use hash::Hash;
pub use message::Message;

#[derive(Debug)]
pub enum Error {
    AmountError,
    CountError,
    EmptyError,
    DuplicateError,
    IndexError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AmountError => "Invalid amount provided.".fmt(f),
            Error::CountError => "Invalid count number provided.".fmt(f),
            Error::DuplicateError => "The object in the set must be unique".fmt(f),
            Error::EmptyError => "The length of the object is empty".fmt(f),
            Error::IndexError => "The position of index is not correct.".fmt(f),
        }
    }
}
