// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Attribute, Field, Fields, Generics, Ident, Index, Token, Type, Variant,
};

const VALID_TAG_TYPES: &[&str] = &["u8", "u16", "u32", "u64"];

struct Fragments {
    pattern: TokenStream,
    pack: TokenStream,
    packed_len: TokenStream,
    unpack: TokenStream,
}

impl Fragments {
    fn new<const NAMED: bool>(name: TokenStream, fields: &Punctuated<Field, Comma>) -> Self {
        let len = fields.len();
        let mut types = Vec::with_capacity(len);
        let mut labels = Vec::with_capacity(len);
        let mut values = Vec::with_capacity(len);

        for (index, Field { ident, ty, .. }) in fields.iter().enumerate() {
            if NAMED {
                // This is a named field, which means its `ident` cannot be `None`
                labels.push(ident.as_ref().unwrap().to_token_stream());
            } else {
                labels.push(proc_macro2::Literal::u64_unsuffixed(index as u64).to_token_stream());
            }
            types.push(ty);
            values.push(format_ident!("field_{}", index));
        }

        Self {
            pattern: quote!(#name { #(#labels: #values),* }),
            pack: quote! {
                #(<#types>::pack(&#values, packer)?;) *
                Ok(())
            },
            packed_len: quote!(0 #(+ <#types>::packed_len(#values))*),
            unpack: quote! {Ok(#name {
                #(#labels: <#types>::unpack(unpacker).map_err(|x| x.coerce())?,)*
            })},
        }
    }

    fn gen_bodies(self) -> (TokenStream, TokenStream, TokenStream) {
        let Self {
            pattern,
            pack,
            packed_len,
            unpack,
        } = self;

        let pack = quote! {
            let #pattern = self;
            #pack
        };

        let packed_len = quote! {
            let #pattern = self;
            #packed_len
        };

        (pack, packed_len, unpack)
    }

    fn gen_branches<Tag: ToTokens, TagTy: ToTokens>(
        self,
        tag: Tag,
        tag_ty: TagTy,
    ) -> (TokenStream, TokenStream, TokenStream) {
        let Self {
            pattern,
            pack,
            packed_len,
            unpack,
        } = self;

        let pack = quote! {
            #pattern => {
                (#tag as #tag_ty).pack(packer)?;
                #pack
            }
        };

        let packed_len = quote!(#pattern => (#tag as #tag_ty).packed_len() + #packed_len);

        let unpack = quote!(#tag => #unpack);

        (pack, packed_len, unpack)
    }
}

pub(crate) fn gen_struct_bodies(struct_fields: Fields) -> (TokenStream, TokenStream, TokenStream) {
    match struct_fields {
        Fields::Named(fields) => Fragments::new::<true>(quote!(Self), &fields.named).gen_bodies(),
        Fields::Unnamed(fields) => Fragments::new::<false>(quote!(Self), &fields.unnamed).gen_bodies(),
        Fields::Unit => (quote!(Ok(())), quote!(0), quote!(Ok(Self))),
    }
}

pub(crate) fn gen_enum_bodies(
    variants: &Punctuated<Variant, Comma>,
    tag_ty: Type,
) -> (TokenStream, TokenStream, TokenStream) {
    let len = variants.len();

    match &tag_ty {
        Type::Path(ty_path) if VALID_TAG_TYPES.iter().any(|ty| ty_path.path.is_ident(ty)) => (),
        _ => abort!(tag_ty.span(), "Tags for enums can only be unisigned, sized integers."),
    }

    let mut indices = Vec::<(Index, &Ident)>::with_capacity(len);

    let mut pack_branches = Vec::with_capacity(len);
    let mut packed_len_branches = Vec::with_capacity(len);
    let mut unpack_branches = Vec::with_capacity(len);

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

        let name = quote!(Self::#ident);

        let (pack_branch, packed_len_branch, unpack_branch) = match fields {
            Fields::Named(fields) => Fragments::new::<true>(name, &fields.named).gen_branches(tag, &tag_ty),
            Fields::Unnamed(fields) => Fragments::new::<false>(name, &fields.unnamed).gen_branches(tag, &tag_ty),
            Fields::Unit => (
                quote!(#name => (#tag as #tag_ty).pack(packer)),
                quote!(#name => (#tag as #tag_ty).packed_len()),
                quote!(#tag => Ok(#name)),
            ),
        };

        pack_branches.push(pack_branch);
        packed_len_branches.push(packed_len_branch);
        unpack_branches.push(unpack_branch);
    }

    (
        quote! {
            match self {
                #(#pack_branches,) *
            }
        },
        quote! {
            match self {
                #(#packed_len_branches,) *
            }
        },
        quote! {
            match unpacker.unpack_infallible::<#tag_ty>()? {
                #(#unpack_branches,) *
                tag => Err(bee_common::packable::UnpackError::Packable(bee_common::packable::UnknownTagError(tag).into()))
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
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics Packable for #ident #ty_generics #where_clause {
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
