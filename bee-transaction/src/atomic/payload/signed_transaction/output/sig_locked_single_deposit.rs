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

pub struct WotsAddress([u8; 49]);

pub struct Ed25519Address([u8; 32]);

pub enum Address {
    Wots(WotsAddress),
    Ed25519(Ed25519Address)
}

pub struct SigLockedSingleDeposit {
    address: Address,
    amount: u64
}
