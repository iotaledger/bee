// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    attribute::{Tag, TagType, UnpackError},
    fragments::Fragments,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{spanned::Spanned, Attribute, DataEnum, DataStruct, DeriveInput, Fields, Generics, Ident, Type, Variant};

pub(crate) struct TraitImpl {
    type_name: Ident,
    generics: Generics,
    unpack_error: TokenStream,
    pack: TokenStream,
    packed_len: TokenStream,
    unpack: TokenStream,
}

impl TraitImpl {
    pub(crate) fn new(input: DeriveInput) -> syn::Result<Self> {
        match input.data {
            syn::Data::Struct(data) => Self::from_struct(data, input.generics, input.attrs, input.ident),
            syn::Data::Enum(data) => Self::from_enum(data, input.generics, input.attrs, input.ident),
            syn::Data::Union(_) => Err(syn::Error::new(
                input.ident.span(),
                "The `Packable` trait cannot be derived for unions.",
            )),
        }
    }

    fn from_enum(data: DataEnum, generics: Generics, attrs: Vec<Attribute>, type_name: Ident) -> syn::Result<Self> {
        let (tag_ty, tag_with_err) = match TagType::new(&attrs)? {
            TagType { ty: Some(ty), with_err } => (ty, with_err),
            TagType { ty: None, with_err } => {
                match attrs.iter().find_map(|attr| {
                    if attr.path.is_ident("repr") {
                        Some(attr.parse_args::<Type>())
                    } else {
                        None
                    }
                }) {
                    Some(ty) => (ty?, with_err),
                    None => {
                        return Err(syn::Error::new(
                            type_name.span(),
                            "Enums that derive `Packable` require a `#[packable(tag_type = ...)]` attribute.",
                        ));
                    }
                }
            }
        };

        let unpack_error_attr = UnpackError::for_enum(&attrs, &tag_ty)?;

        let len = data.variants.len();

        // The names of the types that can be used for tags.
        const VALID_TAG_TYPES: &[&str] = &["u8", "u16", "u32", "u64"];

        // Validate that the tag type is in `VALID_TAG_TYPES`.
        match &tag_ty {
            Type::Path(ty_path) if VALID_TAG_TYPES.iter().any(|ty| ty_path.path.is_ident(ty)) => (),
            ty => {
                let (last, rest) = VALID_TAG_TYPES.split_last().unwrap();
                return Err(syn::Error::new(
                    ty.span(),
                    format!(
                        "Tags for enums can only be of type `{}` or `{}`.",
                        rest.join("`, `"),
                        last
                    ),
                ));
            }
        }

        // Store the tags and names of the variants so we can guarantee that tags are unique.
        let mut tags = Vec::with_capacity(len);

        let mut tag_idents = Vec::with_capacity(len);

        // The branch for packing each variant.
        let mut pack_branches = Vec::with_capacity(len);
        // The branch with the packing length of each variant.
        let mut packed_len_branches = Vec::with_capacity(len);
        // The branch for unpacking each variant.
        let mut unpack_branches = Vec::with_capacity(len);

        for variant in data.variants {
            let Variant {
                attrs,
                ident,
                fields,
                discriminant,
            } = variant;

            let tag = match Tag::new(&attrs)?.value {
                Some(tag) => tag.into_token_stream(),
                None => match discriminant {
                    Some((_, tag)) => tag.into_token_stream(),
                    None => {
                        return Err(syn::Error::new(
                            ident.span(),
                            "All variants of an enum that derives `Packable` require a `#[packable(tag = ...)]` attribute.",
                        ));
                    }
                },
            };

            // @pvdrz: The span here is very important, otherwise the compiler won't detect
            // unreachable patterns in the generated code for some reason. I think this is related
            // to `https://github.com/rust-lang/rust/pull/80632`
            let tag_ident = format_ident!("__TAG_{}", tags.len(), span = tag.span());

            // Add the `Self::` prefix to the name of the variant.
            let name = quote!(Self::#ident);

            let fragments = match fields {
                Fields::Named(fields) => Fragments::new::<true>(name, &fields.named, &unpack_error_attr.with)?,
                Fields::Unnamed(fields) => Fragments::new::<false>(name, &fields.unnamed, &unpack_error_attr.with)?,
                Fields::Unit => Fragments::new::<true>(name, &Default::default(), &unpack_error_attr.with)?,
            };

            let (pack_branch, packed_len_branch, unpack_branch) = fragments.consume_for_variant(&tag_ident, &tag_ty);

            tags.push(tag);
            tag_idents.push(tag_ident);

            pack_branches.push(pack_branch);
            packed_len_branches.push(packed_len_branch);
            unpack_branches.push(unpack_branch);
        }

        let tag_decls = quote!(#(const #tag_idents: #tag_ty = #tags;) *);

        // Add a surrounding match expresison for the branches.
        let pack = quote! {
            #tag_decls
            match self {
                #(#pack_branches,) *
            }
        };
        let packed_len = quote! {
            #tag_decls
            match self {
                #(#packed_len_branches,) *
            }
        };

        let unpack = quote! {
            #tag_decls
            #[deny(unreachable_patterns)]
            match <#tag_ty>::unpack(unpacker).infallible()? {
                #(#unpack_branches,) *
                tag => Err(bee_packable::error::UnpackError::Packable(#tag_with_err(tag).into()))
            }
        };

        Ok(Self {
            type_name,
            generics,
            unpack_error: unpack_error_attr.ty.to_token_stream(),
            pack,
            packed_len,
            unpack,
        })
    }

    fn from_struct(data: DataStruct, generics: Generics, attrs: Vec<Attribute>, type_name: Ident) -> syn::Result<Self> {
        let first_field_ty = || data.fields.iter().next().map(|field| &field.ty);

        let unpack_error = UnpackError::for_struct(&attrs, first_field_ty)?;

        let fragments = match data.fields {
            Fields::Named(fields) => Fragments::new::<true>(quote!(Self), &fields.named, &unpack_error.with)?,
            Fields::Unnamed(fields) => Fragments::new::<false>(quote!(Self), &fields.unnamed, &unpack_error.with)?,
            Fields::Unit => Fragments::new::<true>(quote!(Self), &Default::default(), &unpack_error.with)?,
        };

        let (pack, packed_len, unpack) = fragments.consume_for_struct();

        Ok(Self {
            type_name,
            generics,
            unpack_error: unpack_error.ty.to_token_stream(),
            pack,
            packed_len,
            unpack,
        })
    }
}

impl ToTokens for TraitImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            type_name,
            generics,
            unpack_error,
            pack,
            packed_len,
            unpack,
        } = &self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let impl_tokens = quote! {
            impl #impl_generics Packable for #type_name #ty_generics #where_clause {
                type UnpackError = #unpack_error;

                fn pack<P: bee_packable::packer::Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
                    use bee_packable::coerce::*;
                    #pack
                }

                fn unpack<U: bee_packable::unpacker::Unpacker>(unpacker: &mut U) -> Result<Self, bee_packable::error::UnpackError<Self::UnpackError, U::Error>> {
                    use bee_packable::coerce::*;
                    #unpack
                }

                fn packed_len(&self) -> usize {
                    #packed_len
                }
            }
        };

        impl_tokens.to_tokens(tokens);
    }
}
