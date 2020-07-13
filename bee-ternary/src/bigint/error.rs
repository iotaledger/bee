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

//! Big integers errors.

use thiserror::Error;

/// Errors related to big integers.
#[derive(Clone, Debug, Error)]
pub enum Error {
    /// Error when converting and binary representation exceeds ternary range.
    #[error("Binary representation exceeds ternary range.")]
    BinaryExceedsTernaryRange,
    /// Error when converting and ternary representation exceeds binary range.
    #[error("Ternary representation exceeds binary range.")]
    TernaryExceedsBinaryRange,
}
