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

use crate::{
    atomic::{payload::Payload, Hash},
    Vertex,
};

pub struct Message {
    pub trunk: Hash,
    pub branch: Hash,
    pub payload: Payload,
    pub nonce: u64,
}

impl Vertex for Message {
    type Hash = Hash;

    fn trunk(&self) -> &Self::Hash {
        &self.trunk
    }

    fn branch(&self) -> &Self::Hash {
        &self.branch
    }
}
