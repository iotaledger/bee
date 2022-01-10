// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    enum_info::EnumInfo, fragments::Fragments, struct_info::StructInfo, tag_type_info::TagTypeInfo,
    variant_info::VariantInfo,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{spanned::Spanned, Data, DeriveInput, Generics, Ident};

pub(crate) struct TraitImpl {
    ident: Ident,
    generics: Generics,
    unpack_error: TokenStream,
    pack: TokenStream,
    unpack: TokenStream,
    crate_name: Ident,
}

impl TraitImpl {
    pub(crate) fn new(input: DeriveInput, crate_name: Ident) -> syn::Result<Self> {
        match input.data {
            Data::Struct(data) => {
                let info = StructInfo::new(input.ident.clone().into(), &data.fields, &input.attrs, &crate_name)?;

                let unpack_error = info.unpack_error.unpack_error.clone().into_token_stream();

                let Fragments { pattern, pack, unpack }: Fragments = Fragments::new(info.inner, &crate_name);

                Ok(Self {
                    ident: input.ident,
                    generics: input.generics,
                    unpack_error,
                    pack: quote! {
                        let #pattern = self;
                        #pack
                    },
                    unpack,
                    crate_name,
                })
            }
            Data::Enum(data) => {
                let enum_ident = &input.ident;
                let info = EnumInfo::new(enum_ident.clone(), data, &input.attrs, &crate_name)?;

                let TagTypeInfo {
                    tag_type,
                    with_error: tag_with_error,
                } = info.tag_type;

                let unpack_error = info.unpack_error.unpack_error.into_token_stream();

                let len = info.variants_info.len();
                let mut pack_arms = Vec::with_capacity(len);
                let mut unpack_arms = Vec::with_capacity(len);
                let mut tag_decls = Vec::with_capacity(len);
                let mut tag_variants_and_idents = Vec::with_capacity(len);

                for (index, VariantInfo { tag, inner }) in info.variants_info.into_iter().enumerate() {
                    let variant_ident = inner.path.segments.last().unwrap().clone();

                    let Fragments { pattern, pack, unpack } = Fragments::new(inner, &crate_name);

                    // @pvdrz: The span here is very important, otherwise the compiler won't detect
                    // unreachable patterns in the generated code for some reason. I think this is related
                    // to `https://github.com/rust-lang/rust/pull/80632`
                    let tag_ident = format_ident!("__TAG_{}", index, span = tag.span());

                    pack_arms.push(quote!(#pattern => {
                        <#tag_type as #crate_name::Packable>::pack(&#tag, packer)?;
                        #pack
                    }));

                    unpack_arms.push(quote!(#tag_ident => {
                        #unpack
                    }));

                    tag_decls.push(quote!(const #tag_ident: #tag_type = #tag;));

                    tag_variants_and_idents.push((tag_ident, variant_ident));
                }

                let mut tag_asserts = Vec::with_capacity(len * (len - 1) / 2);

                for (index, (fst, fst_variant)) in tag_variants_and_idents.iter().enumerate() {
                    if let Some(idents_and_variants) = tag_variants_and_idents.get((index + 1)..) {
                        for (snd, snd_variant) in idents_and_variants {
                            let tag_assert = quote!(
                                const _: () = assert!(#fst != #snd, concat!("The tags for the variants `", stringify!(#fst_variant), "` and `", stringify!(#snd_variant) ,"` of enum `", stringify!(#enum_ident), "` are equal"));
                            );
                            tag_asserts.push(tag_assert);
                        }
                    }
                }

                Ok(Self {
                    ident: enum_ident.clone(),
                    generics: input.generics,
                    unpack_error,
                    pack: quote!(match self {
                        #(#pack_arms)*
                    }),
                    unpack: quote! {
                        #(#tag_decls)*
                        #(#tag_asserts)*

                        match <#tag_type as #crate_name::Packable>::unpack::<_, VERIFY>(unpacker).infallible()? {
                            #(#unpack_arms)*
                            tag => Err(#crate_name::error::UnpackError::from_packable(#tag_with_error(tag)))
                        }
                    },
                    crate_name,
                })
            }
            Data::Union(_) => Err(syn::Error::new(
                input.ident.span(),
                "The `Packable` trait cannot be derived for unions.",
            )),
        }
    }
}

impl ToTokens for TraitImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            ident: type_name,
            generics,
            unpack_error,
            pack,
            unpack,
            crate_name,
        } = &self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let impl_tokens = quote! {
            impl #impl_generics #crate_name::Packable for #type_name #ty_generics #where_clause {
                type UnpackError = #unpack_error;

                fn pack<P: #crate_name::packer::Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
                    use #crate_name::error::UnpackErrorExt;
                    #pack
                }

                fn unpack<U: #crate_name::unpacker::Unpacker, const VERIFY: bool>(unpacker: &mut U) -> Result<Self, #crate_name::error::UnpackError<Self::UnpackError, U::Error>> {
                    use #crate_name::error::UnpackErrorExt;
                    #unpack
                }
            }
        };

        impl_tokens.to_tokens(tokens);
    }
}
