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

use crate::ternary::{constants::MESSAGE_FRAGMENT_LENGTH, wots::WotsSecurityLevel};

use bee_crypto::ternary::HASH_LENGTH;
use bee_ternary::{T1B1Buf, T3B1Buf, TritBuf, Trits, Tryte, T1B1, T3B1};

use thiserror::Error;

use std::convert::TryFrom;

/// Errors occuring during normalization.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// Invalid message length.
    #[error("Invalid message length.")]
    InvalidMessageLength,
}

/// When applying WOTS on a non-normalized message, the amount of private key data leaked is not uniform and some
/// messages could result in most of (or all of) the key being leaked. As a consequence, even after one signature there
/// is a varying chance that brute forcing another message becomes feasible. By normalizing the message, such "extreme"
/// cases get alleviated, so that every signature exactly leaks half of the private key.
pub fn normalize(message: &Trits<T1B1>) -> Result<TritBuf<T1B1Buf>, Error> {
    if message.len() != HASH_LENGTH {
        return Err(Error::InvalidMessageLength);
    }

    let mut normalized = [0i8; WotsSecurityLevel::High as usize * MESSAGE_FRAGMENT_LENGTH];

    for i in 0..WotsSecurityLevel::High as usize {
        let mut sum: i16 = 0;

        for j in (i * MESSAGE_FRAGMENT_LENGTH)..((i + 1) * MESSAGE_FRAGMENT_LENGTH) {
            // Safe to unwrap because 3 trits can't underflow/overflow an i8.
            normalized[j] = i8::try_from(&message[j * 3..j * 3 + 3]).unwrap();
            sum += normalized[j] as i16;
        }

        if sum > 0 {
            while sum > 0 {
                for t in &mut normalized[i * MESSAGE_FRAGMENT_LENGTH..(i + 1) * MESSAGE_FRAGMENT_LENGTH] {
                    if (*t as i8) > Tryte::MIN_VALUE as i8 {
                        *t -= 1;
                        break;
                    }
                }
                sum -= 1;
            }
        } else {
            while sum < 0 {
                for t in &mut normalized[i * MESSAGE_FRAGMENT_LENGTH..(i + 1) * MESSAGE_FRAGMENT_LENGTH] {
                    if (*t as i8) < Tryte::MAX_VALUE as i8 {
                        *t += 1;
                        break;
                    }
                }
                sum += 1;
            }
        }
    }

    // This usage of unsafe is fine since we are creating the normalized trits inside this function and we know that the
    // content can't go wrong.
    Ok(unsafe {
        Trits::<T3B1>::from_raw_unchecked(&normalized, normalized.len() * 3)
            .to_buf::<T3B1Buf>()
            .encode::<T1B1Buf>()
    })
}
