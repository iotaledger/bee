// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use proc_macro::{self, TokenStream};
use syn::{parse_macro_input, Data, DeriveInput, Type};

mod packable;

#[proc_macro_derive(Packable, attributes(packable))]
pub fn packable(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, attrs, .. } = parse_macro_input!(input);

    let (pack, unpack) = match data {
        Data::Struct(data_struct) => packable::gen_struct_bodies(data_struct.fields),
        Data::Enum(data_enum) => {
            let ty = packable::parse_attr::<Type>("ty", &attrs).unwrap();
            packable::gen_enum_bodies(data_enum.variants.iter(), ty)
        }
        Data::Union(..) => {
            panic!("Unions cannot derive `Packable`")
        }
    };

    packable::gen_impl(&ident, pack, unpack).into()
}
