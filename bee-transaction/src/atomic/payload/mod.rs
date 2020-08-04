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

mod indexation;
mod milestone;
mod signed_data;
pub mod signed_transaction;
mod unsigned_data;

pub use indexation::Indexation;
pub use milestone::Milestone;
pub use signed_data::SignedData;
pub use signed_transaction::SignedTransaction;
pub use unsigned_data::UnsignedData;

pub enum Payload {
    Indexation(Indexation),
    Milestone(Milestone),
    SignedData(SignedData),
    SignedTransaction(SignedTransaction),
    UnsignedData(UnsignedData),
}
