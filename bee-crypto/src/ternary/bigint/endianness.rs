// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Endianness markers for big integers.

/// Big endian marker.
#[derive(Clone, Copy, Debug)]
pub struct BigEndian {}

/// Little endian marker.
#[derive(Clone, Copy, Debug)]
pub struct LittleEndian {}
