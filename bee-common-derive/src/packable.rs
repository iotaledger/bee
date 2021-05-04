// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Fields, FieldsNamed, FieldsUnnamed, Ident, Index, Token, Type, Variant,
};

pub(crate) fn gen_struct_bodies(struct_fields: Fields) -> (TokenStream, TokenStream) {
    let (pack, unpack);

    match struct_fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            let (labels, types): (Vec<&Ident>, Vec<&Type>) = named
                .iter()
                .map(|field| (field.ident.as_ref().unwrap(), &field.ty))
                .unzip();

            pack = quote! {
                #(self.#labels.pack(packer)?;) *
                Ok(())
            };

            unpack = quote! {
                Ok(Self { #(#labels: <#types>::unpack(unpacker)?,)* })
            };
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let (indices, types): (Vec<Index>, Vec<&Type>) = unnamed
                .iter()
                .enumerate()
                .map(|(index, field)| (index.into(), &field.ty))
                .unzip();

            pack = quote! {
                #(self.#indices.pack(packer)?;) *
                Ok(())
            };

            unpack = quote! {
                Ok(Self(#(<#types>::unpack(unpacker)?), *))
            };
        }
        Fields::Unit => {
            pack = quote!(Ok(()));
            unpack = quote!(Ok(Self));
        }
    }

    (pack, unpack)
}

pub(crate) fn gen_enum_bodies<'a>(
    variants: impl Iterator<Item = &'a Variant> + 'a,
    ty: Type,
) -> (TokenStream, TokenStream) {
    let mut indices = Vec::new();

    let (pack_branches, unpack_branches): (Vec<TokenStream>, Vec<TokenStream>) = variants
        .map(
            |Variant {
                 attrs, ident, fields, ..
             }| {
                let index = parse_attr::<Index>("id", attrs).unwrap().index as u64;

                match indices.binary_search(&index) {
                    Ok(_) => panic!("The ID {} is already being used.", index),
                    Err(pos) => indices.insert(pos, index),
                }

                let id = proc_macro2::Literal::u64_unsuffixed(index);

                let pack_branch: TokenStream;
                let unpack_branch: TokenStream;

                match fields {
                    Fields::Named(FieldsNamed { named, .. }) => {
                        let (labels, types): (Vec<&Ident>, Vec<&Type>) = named
                            .iter()
                            .map(|field| (field.ident.as_ref().unwrap(), &field.ty))
                            .unzip();

                        pack_branch = quote! {
                            Self::#ident{#(#labels), *} => {
                                (#id as #ty).pack(packer)?;
                                #(#labels.pack(packer)?;) *
                                Ok(())
                            }
                        };

                        unpack_branch = quote! {
                            #id => Ok(Self::#ident{
                                #(#labels: <#types>::unpack(unpacker)?,) *
                            })
                        };
                    }
                    Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let (fields, types): (Vec<Ident>, Vec<&Type>) = unnamed
                            .iter()
                            .enumerate()
                            .map(|(index, field)| {
                                (
                                    Ident::new(&format!("field_{}", index), proc_macro2::Span::call_site()),
                                    &field.ty,
                                )
                            })
                            .unzip();

                        pack_branch = quote! {
                            Self::#ident(#(#fields), *) => {
                                (#id as #ty).pack(packer)?;
                                #(#fields.pack(packer)?;) *
                                Ok(())
                            }
                        };

                        unpack_branch = quote! {
                            #id => Ok(Self::#ident(
                                #(<#types>::unpack(unpacker)?), *
                            ))
                        };
                    }
                    Fields::Unit => {
                        pack_branch = quote! {
                            Self::#ident => (#id as #ty).pack(packer)
                        };

                        unpack_branch = quote! {
                            #id => Ok(Self::#ident)
                        };
                    }
                };
                (pack_branch, unpack_branch)
            },
        )
        .unzip();

    (
        quote! {
            match self {
                #(#pack_branches,) *
            }
        },
        quote! {
            match <#ty>::unpack(unpacker)? {
                #(#unpack_branches,) *
                id => Err(U::Error::invalid_variant(id as u64))
            }
        },
    )
}

pub(crate) fn gen_impl(ident: &Ident, pack_body: TokenStream, unpack_body: TokenStream) -> TokenStream {
    quote! {
        impl Packable for #ident {
            fn pack<P: bee_common::packable::Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
                #pack_body
            }
            fn unpack<U: bee_common::packable::Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
                #unpack_body
            }
        }
    }
}

pub(crate) fn parse_attr<T: Parse>(ident: &str, attrs: &[Attribute]) -> syn::Result<T> {
    struct AttrArg<T> {
        ident: Ident,
        value: T,
    }

    impl<T: Parse> Parse for AttrArg<T> {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let ident = input.parse::<Ident>()?;
            let _ = input.parse::<Token![=]>()?;
            let value = input.parse::<T>()?;

            Ok(Self { ident, value })
        }
    }

    for attr in attrs {
        if attr.path.is_ident("packable") {
            let arg = attr.parse_args::<AttrArg<T>>()?;

            if arg.ident != ident {
                panic!("Expected argument `{}` found `{}`.", ident, arg.ident);
            }

            return Ok(arg.value);
        }
    }

    panic!("There is no `packable` attribute with a `{}` argument.", ident)
}
