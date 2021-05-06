// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote};
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
                // This is a named field, which means its `ident` cannot be `None`
                .map(|field| (field.ident.as_ref().unwrap(), &field.ty))
                .unzip();

            pack = quote! {
                #(<#types>::pack(&self.#labels, packer)?;) *
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
                #(<#types>::pack(&self.#indices, packer)?;) *
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
    let mut indices = Vec::<(Index, &Ident)>::new();

    let (pack_branches, unpack_branches): (Vec<TokenStream>, Vec<TokenStream>) = variants
        .map(
            |Variant {
                 attrs, ident, fields, ..
             }| {
                let curr_index = parse_attr::<Index>("id", attrs).unwrap_or_else(|| {
                    abort!(
                        ident.span(),
                        "All variants of an enum that derives `Packable` require a `#[packable(id = ...)]` attribute."
                    )
                });

                match indices.binary_search_by(|(index, _)| index.index.cmp(&curr_index.index)) {
                    Ok(pos) => abort!(
                        curr_index.span,
                        "The identifier `{}` is already being used for the `{}` variant.",
                        curr_index.index,
                        indices[pos].1
                    ),
                    Err(pos) => indices.insert(pos, (curr_index.clone(), ident)),
                }

                let id = proc_macro2::Literal::u64_unsuffixed(curr_index.index as u64);

                let pack_branch: TokenStream;
                let unpack_branch: TokenStream;

                match fields {
                    Fields::Named(FieldsNamed { named, .. }) => {
                        let (labels, types): (Vec<&Ident>, Vec<&Type>) = named
                            .iter()
                            // This is a named field, which means its `ident` cannot be `None`
                            .map(|field| (field.ident.as_ref().unwrap(), &field.ty))
                            .unzip();

                        pack_branch = quote! {
                            Self::#ident{#(#labels), *} => {
                                (#id as #ty).pack(packer)?;
                                #(<#types>::pack(&#labels, packer)?;) *
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
                            .map(|(index, field)| (format_ident!("field_{}", index), &field.ty))
                            .unzip();

                        pack_branch = quote! {
                            Self::#ident(#(#fields), *) => {
                                (#id as #ty).pack(packer)?;
                                #(<#types>::pack(&#fields, packer)?;) *
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

pub(crate) fn parse_attr<T: Parse>(ident: &str, attrs: &[Attribute]) -> Option<T> {
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
            match attr.parse_args::<AttrArg<T>>() {
                Ok(arg) if arg.ident == ident => return Some(arg.value),
                _ => (),
            }
        }
    }

    None
}
