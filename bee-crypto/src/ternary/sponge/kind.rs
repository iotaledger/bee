// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// The different kinds of Sponges.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SpongeKind {
    /// Kerl.
    Kerl,
    /// CurlP with 27 rounds.
    CurlP27,
    /// CurlP with 81 rounds.
    CurlP81,
    /// Undrolled CurlP with 81 rounds.
    UnrolledCurlP81,
}
