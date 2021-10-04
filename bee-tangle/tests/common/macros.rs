// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[macro_export]
macro_rules! build_tangle {
    ($builder:expr, $node:expr => $($parent:expr),* ; $($tail:tt)*) => {
        $builder.add_node($node, [$($parent),*]);
        build_tangle!{$builder, $($tail)*}
    };

    ($builder:expr,) => {
        $builder.build()
    }
}

#[macro_export]
macro_rules! tangle {
    ($($tail:tt)*) => {
        {
            let mut builder = common::TangleBuilder::new();
            build_tangle!{builder, $($tail)*}
        }
    }
}
