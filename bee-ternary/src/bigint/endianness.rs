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

//! Endianness markers for big integers.

use crate::bigint::sealed::Sealed;

trait Endianness: Sealed {}

/// Big endian marker.
#[derive(Clone, Copy, Debug)]
pub struct BigEndian {}

impl Sealed for BigEndian {}
impl Endianness for BigEndian {}

/// Little endian marker.
#[derive(Clone, Copy, Debug)]
pub struct LittleEndian {}

impl Sealed for LittleEndian {}
impl Endianness for LittleEndian {}
