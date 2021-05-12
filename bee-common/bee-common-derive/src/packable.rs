// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, Fields, FieldsNamed, FieldsUnnamed, Generics, Ident, Index, Token, Type, TypePath, Variant,
};

pub(crate) fn gen_struct_bodies(struct_fields: Fields) -> (TokenStream, TokenStream, TokenStream) {
    let (pack, unpack, packed_len);

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
                Ok(Self { #(#labels: <#types>::unpack(unpacker).map_err(bee_common::packable::UnpackError::coerce)?,)* })
            };

            packed_len = quote! {
                0 #(+ <#types>::packed_len(&self.#labels)) *
            }
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
                Ok(Self(#(<#types>::unpack(unpacker).map_err(bee_common::packable::UnpackError::coerce)?), *))
            };

            packed_len = quote! {
                0 #(+ <#types>::packed_len(&self.#indices)) *
            }
        }
        Fields::Unit => {
            pack = quote!(Ok(()));
            unpack = quote!(Ok(Self));
            packed_len = quote!(0);
        }
    }

    (pack, unpack, packed_len)
}

pub(crate) fn gen_enum_bodies<'a>(
    variants: impl Iterator<Item = &'a Variant> + 'a,
    ty: Type,
) -> (TokenStream, TokenStream, TokenStream) {
    const TYPES: &[&str] = &["u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64"];

    match &ty {
        Type::Path(TypePath { path, .. }) if TYPES.iter().any(|ty| path.is_ident(ty)) => (),
        _ => abort!(ty.span(), "Tags for enums can only be sized integers."),
    }

    let mut indices = Vec::<(Index, &Ident)>::new();

    let mut pack_branches = Vec::new();
    let mut unpack_branches = Vec::new();
    let mut packed_len_branches = Vec::new();

    for variant in variants {
        let Variant {
            attrs, ident, fields, ..
        } = variant;

        let curr_index = parse_attr::<Index>("tag", attrs).unwrap_or_else(|| {
            abort!(
                ident.span(),
                "All variants of an enum that derives `Packable` require a `#[packable(tag = ...)]` attribute."
            )
        });

        match indices.binary_search_by(|(index, _)| index.index.cmp(&curr_index.index)) {
            Ok(pos) => abort!(
                curr_index.span,
                "The tag `{}` is already being used for the `{}` variant.",
                curr_index.index,
                indices[pos].1
            ),
            Err(pos) => indices.insert(pos, (curr_index.clone(), ident)),
        }

        let tag = proc_macro2::Literal::u64_unsuffixed(curr_index.index as u64);

        let pack_branch: TokenStream;
        let unpack_branch: TokenStream;
        let packed_len_branch: TokenStream;

        match fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let (labels, types): (Vec<&Ident>, Vec<&Type>) = named
                    .iter()
                    // This is a named field, which means its `ident` cannot be `None`
                    .map(|field| (field.ident.as_ref().unwrap(), &field.ty))
                    .unzip();

                pack_branch = quote! {
                    Self::#ident{#(#labels), *} => {
                        (#tag as #ty).pack(packer)?;
                        #(<#types>::pack(&#labels, packer)?;) *
                        Ok(())
                    }
                };

                unpack_branch = quote! {
                    #tag => Ok(Self::#ident{
                        #(#labels: <#types>::unpack(unpacker).map_err(bee_common::packable::UnpackError::coerce)?,) *
                    })
                };

                packed_len_branch = quote! {
                    Self::#ident{#(#labels), *} => {
                        (#tag as #ty).packed_len() #(+ <#types>::packed_len(&#labels)) *
                    }
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
                        (#tag as #ty).pack(packer)?;
                        #(<#types>::pack(&#fields, packer)?;) *
                        Ok(())
                    }
                };

                unpack_branch = quote! {
                    #tag => Ok(Self::#ident(
                        #(<#types>::unpack(unpacker).map_err(bee_common::packable::UnpackError::coerce)?), *
                    ))
                };

                packed_len_branch = quote! {
                    Self::#ident(#(#fields), *) => {
                        (#tag as #ty).packed_len() #(+ <#types>::packed_len(&#fields)) *
                    }
                };
            }
            Fields::Unit => {
                pack_branch = quote! {
                    Self::#ident => (#tag as #ty).pack(packer)
                };

                unpack_branch = quote! {
                    #tag => Ok(Self::#ident)
                };

                packed_len_branch = quote! {
                    Self::#ident => (#tag as #ty).packed_len()
                };
            }
        };
        pack_branches.push(pack_branch);
        unpack_branches.push(unpack_branch);
        packed_len_branches.push(packed_len_branch);
    }

    (
        quote! {
            match self {
                #(#pack_branches,) *
            }
        },
        quote! {
            match unpacker.unpack_infallible::<#ty>()? {
                #(#unpack_branches,) *
                id => Err(bee_common::packable::UnpackError::Packable(bee_common::packable::UnknownTagError(id).into()))
            }
        },
        quote! {
            match self {
                #(#packed_len_branches,) *
            }
        },
    )
}

pub(crate) fn gen_impl(
    ident: &Ident,
    generics: &Generics,
    error_type: TokenStream,
    pack_body: TokenStream,
    unpack_body: TokenStream,
    packed_len_body: TokenStream,
) -> TokenStream {
    let params = generics.params.iter();

    quote! {
        impl <#(#params: Packable,) *> Packable for #ident #generics {
            type Error = #error_type;

            fn pack<P: bee_common::packable::Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
                #pack_body
            }

            fn unpack<U: bee_common::packable::Unpacker>(unpacker: &mut U) -> Result<Self, bee_common::packable::UnpackError<Self::Error, U::Error>> {
                #unpack_body
            }

            fn packed_len(&self) -> usize {
                #packed_len_body
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
