// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    attribute::{PackError, Tag, TagType, UnpackError},
    fragments::Fragments,
};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Attribute, DataEnum, DataStruct, DeriveInput, Fields, Generics, Ident, Type, Variant};

pub(crate) struct TraitImpl {
    type_name: Ident,
    generics: Generics,
    pack_error: TokenStream,
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
        let first_field_ty = || {
            data.variants
                .first()
                .and_then(|variant| variant.fields.iter().next().map(|field| &field.ty))
        };

        let TagType {
            ty: tag_ty,
            with_err: tag_with_err,
        } = TagType::new(&attrs, &type_name)?;

        let pack_error_attr = PackError::new(&attrs, &first_field_ty)?;

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

        // The branch for packing each variant.
        let mut pack_branches = Vec::with_capacity(len);
        // The branch with the packing length of each variant.
        let mut packed_len_branches = Vec::with_capacity(len);
        // The branch for unpacking each variant.
        let mut unpack_branches = Vec::with_capacity(len);

        for variant in data.variants {
            let Variant {
                attrs, ident, fields, ..
            } = variant;

            let Tag { value: tag } = Tag::new(&attrs, &type_name)?;

            tags.push(tag.clone());

            // Add the `Self::` prefix to the name of the variant.
            let name = quote!(Self::#ident);

            let fragments = match fields {
                Fields::Named(fields) => {
                    Fragments::new::<true>(name, &fields.named, &pack_error_attr.with, &unpack_error_attr.with)?
                }
                Fields::Unnamed(fields) => {
                    Fragments::new::<false>(name, &fields.unnamed, &pack_error_attr.with, &unpack_error_attr.with)?
                }
                Fields::Unit => Fragments::new::<true>(
                    name,
                    &Default::default(),
                    &pack_error_attr.with,
                    &unpack_error_attr.with,
                )?,
            };

            let (pack_branch, packed_len_branch, unpack_branch) = fragments.consume_for_variant(&tag, &tag_ty);

            pack_branches.push(pack_branch);
            packed_len_branches.push(packed_len_branch);
            unpack_branches.push(unpack_branch);
        }

        // Add a surrounding match expresison for the branches.
        let pack = quote! {
            match self {
                #(#pack_branches,) *
            }
        };
        let packed_len = quote! {
            match self {
                #(#packed_len_branches,) *
            }
        };

        let unpack = quote! {
            #[deny(unreachable_patterns)]
            match <#tag_ty>::unpack(unpacker).infallible()? {
                #(#unpack_branches,) *
                tag => Err(bee_packable::error::UnpackError::Packable(#tag_with_err(tag).into()))
            }
        };

        Ok(Self {
            type_name,
            generics,
            pack_error: pack_error_attr.ty.to_token_stream(),
            unpack_error: unpack_error_attr.ty.to_token_stream(),
            pack,
            packed_len,
            unpack,
        })
    }

    fn from_struct(data: DataStruct, generics: Generics, attrs: Vec<Attribute>, type_name: Ident) -> syn::Result<Self> {
        let first_field_ty = || data.fields.iter().next().map(|field| &field.ty);

        let pack_error = PackError::new(&attrs, &first_field_ty)?;

        let unpack_error = UnpackError::for_struct(&attrs, first_field_ty)?;

        let (pack, packed_len, unpack) = match data.fields {
            Fields::Named(fields) => {
                Fragments::new::<true>(quote!(Self), &fields.named, &pack_error.with, &unpack_error.with)?
                    .consume_for_struct()
            }
            Fields::Unnamed(fields) => {
                Fragments::new::<false>(quote!(Self), &fields.unnamed, &pack_error.with, &unpack_error.with)?
                    .consume_for_struct()
            }
            Fields::Unit => (quote!(Ok(())), quote!(0), quote!(Ok(Self))),
        };

        Ok(Self {
            type_name,
            generics,
            pack_error: pack_error.ty.to_token_stream(),
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
            pack_error,
            unpack_error,
            pack,
            packed_len,
            unpack,
        } = &self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let impl_tokens = quote! {
            impl #impl_generics Packable for #type_name #ty_generics #where_clause {
                type PackError = #pack_error;
                type UnpackError = #unpack_error;

                fn pack<P: bee_packable::packer::Packer>(&self, packer: &mut P) -> Result<(), bee_packable::error::PackError<Self::PackError, P::Error>> {
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
