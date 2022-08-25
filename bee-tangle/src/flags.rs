// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::convert::Infallible;

use bitflags::bitflags;
use packable::{error::UnpackError, packer::Packer, unpacker::Unpacker, Packable};
use serde::Serialize;

bitflags! {
    /// Flags representing the state of a block.
    #[derive(Default, Serialize)]
    pub struct Flags: u8 {
        /// The block is solid.
        const SOLID = 0b0000_0001;
        /// The block is a milestone.
        const MILESTONE = 0b0000_0010;
        /// The block has been referenced by a milestone.
        const REFERENCED = 0b0000_0100;
        /// The block is valid.
        const VALID = 0b0000_1000;
        /// The block was requested.
        const REQUESTED = 0b0001_0000;
    }
}

impl Flags {
    /// Return whether the flags indicate that the block is solid.
    pub fn is_solid(&self) -> bool {
        self.contains(Flags::SOLID)
    }

    /// Set the solid flag for this block.
    pub fn set_solid(&mut self, is_solid: bool) {
        self.set(Flags::SOLID, is_solid);
    }

    /// Return whether the flags indicate that the block is a milestone.
    pub fn is_milestone(&self) -> bool {
        self.contains(Flags::MILESTONE)
    }

    /// Set the milestone flag for this block.
    pub fn set_milestone(&mut self, is_milestone: bool) {
        self.set(Flags::MILESTONE, is_milestone);
    }

    /// Return whether the flags indicate that the block is referenced.
    pub fn is_referenced(&self) -> bool {
        self.contains(Flags::REFERENCED)
    }

    /// Set the referenced flag for this block.
    pub fn set_referenced(&mut self, is_referenced: bool) {
        self.set(Flags::REFERENCED, is_referenced);
    }

    /// Return whether the flags indicate that the block is valid.
    pub fn is_valid(&self) -> bool {
        self.contains(Flags::VALID)
    }

    /// Set the valid flag for this block.
    pub fn set_valid(&mut self, is_valid: bool) {
        self.set(Flags::VALID, is_valid);
    }

    /// Return whether the flags indicate that the block was requested.
    pub fn was_requested(&self) -> bool {
        self.contains(Flags::REQUESTED)
    }

    /// Set the valid flag for this block.
    pub fn set_requested(&mut self, was_requested: bool) {
        self.set(Flags::REQUESTED, was_requested);
    }
}

impl Packable for Flags {
    type UnpackError = Infallible;
    type UnpackVisitor = ();

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.bits().pack(packer)
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
        visitor: &Self::UnpackVisitor,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        // Flags are only expected to be unpacked from a trusted storage source.
        Ok(unsafe { Self::from_bits_unchecked(u8::unpack::<_, VERIFY>(unpacker, visitor)?) })
    }
}
