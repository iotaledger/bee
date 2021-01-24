// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::{Packable, Read, Write};

use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct Flags: u8 {
        const SOLID = 0b0000_0001;
        const MILESTONE = 0b0000_0010;
        const CONFIRMED = 0b0000_0100;
        const VALID = 0b0000_1000;
    }
}

impl Flags {
    pub fn is_solid(&self) -> bool {
        self.contains(Flags::SOLID)
    }

    pub fn set_solid(&mut self, is_solid: bool) {
        self.set(Flags::SOLID, is_solid);
    }

    pub fn is_milestone(&self) -> bool {
        self.contains(Flags::MILESTONE)
    }

    pub fn set_milestone(&mut self, is_milestone: bool) {
        self.set(Flags::MILESTONE, is_milestone);
    }

    pub fn is_confirmed(&self) -> bool {
        self.contains(Flags::CONFIRMED)
    }

    pub fn set_confirmed(&mut self, is_confirmed: bool) {
        self.set(Flags::CONFIRMED, is_confirmed);
    }

    pub fn is_valid(&self) -> bool {
        self.contains(Flags::VALID)
    }

    pub fn set_valid(&mut self, is_valid: bool) {
        self.set(Flags::VALID, is_valid);
    }
}

impl Packable for Flags {
    type Error = std::io::Error;

    fn packed_len(&self) -> usize {
        self.bits().packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.bits().pack(writer)
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        // Flags are only expected to be unpacked from a trusted storage source.
        Ok(unsafe { Self::from_bits_unchecked(u8::unpack(reader)?) })
    }
}
