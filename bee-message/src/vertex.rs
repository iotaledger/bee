// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub trait Vertex {
    type Id;

    fn parent1(&self) -> &Self::Id;

    fn parent2(&self) -> &Self::Id;
}
