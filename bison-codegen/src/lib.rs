extern crate proc_macro;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::*;

#[proc_macro_derive(Context, attributes(cx))]
pub fn derive_context(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;

    let strukt = match &input.data {
        Data::Struct(s) => s,
        Data::Enum(_) => return Err(Error::new_spanned(input, "expected struct, found enum")),
        Data::Union(_) => return Err(Error::new_spanned(input, "expected struct, found union")),
    };

    let fields = match &strukt.fields {
        Fields::Named(fields) => fields,
        Fields::Unnamed(_) => {
            return Err(Error::new_spanned(
                &strukt.fields,
                "tuple structs are not supported",
            ))
        }
        Fields::Unit => {
            return Err(Error::new_spanned(
                &strukt.fields,
                "unit structs are not supported",
            ))
        }
    };

    let mut ty_lifetime = None;

    for param in &input.generics.params {
        match param {
            GenericParam::Type(_) => {
                return Err(Error::new_spanned(
                    &input.generics.params,
                    "generics type parameters are not supported",
                ))
            }
            GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => {
                if ty_lifetime.is_some() {
                    return Err(Error::new_spanned(
                        &input.generics.params,
                        "only one lifetime parameter is supported",
                    ));
                }

                ty_lifetime = Some(lifetime.clone());
            }
            GenericParam::Const(_) => {
                return Err(Error::new_spanned(
                    &input.generics.params,
                    "const generic parameters are not supported",
                ))
            }
        }
    }

    let fields = fields
        .named
        .iter()
        .map(|field| {
            let name = &field.ident.as_ref().unwrap();
            let extracted = extract(field)?;
            Ok(quote! {
                #name: { #extracted },
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let ty = match ty_lifetime {
        Some(ref l) => quote! { #name<#l> },
        None => quote! { #name },
    };

    let (_, _, where_clause) = input.generics.split_for_impl();

    let context = quote! {
        impl<'req> ::bison::Context<'req> for #ty #where_clause {
            type Future = ::bison::bounded::BoxFuture<'req, Result<Self, ::bison::Error>>;

            fn extract(req: &'req ::bison::http::Request) ->  Self::Future {
                Box::pin(async move { Ok(#name { #(#fields)* }) })
            }
        }
    };

    let with_context = match ty_lifetime {
        Some(_) => quote! {
            impl<'req, 'any> ::bison::WithContext<'req> for #name<'any> #where_clause {
                type Context = #name<'req>;
            }
        },

        None => quote! {
            impl<'req> ::bison::WithContext<'req> for #name #where_clause {
                type Context = #name;
            }
        },
    };

    Ok(quote! {
        #context
        #with_context
    })
}

fn extract(field: &Field) -> Result<TokenStream> {
    let field_name = field.ident.as_ref().unwrap().to_string();
    let ty = &field.ty;

    for attr in &field.attrs {
        let meta = match attr.parse_meta()? {
            Meta::List(list) => list,
            _ => return Err(bad_extractor(&attr)),
        };

        let ident = meta
            .path
            .get_ident()
            .map(Ident::to_string)
            .ok_or(bad_extractor(&attr))?;

        if ident != "cx" || meta.nested.len() != 1 {
            return Err(bad_extractor(meta));
        }

        let (extractor, param) = match meta.nested.first().unwrap() {
            NestedMeta::Meta(Meta::NameValue(nv)) => (&nv.path, Some(&nv.lit)),
            NestedMeta::Meta(Meta::Path(path)) => (path, None),
            _ => return Err(bad_extractor(attr)),
        };

        let param = match param {
            Some(param) => {
                quote_spanned! { ty.span() => ::bison::extract::arg::Argument::new(#field_name, #param) }
            }
            None => {
                quote_spanned! { ty.span() => ::bison::extract::arg::DefaultArgument::new(#field_name) }
            }
        };

        return Ok(quote_spanned! { ty.span() =>
            let result: ::std::result::Result<<#ty as ::bison::extract::Transform<_>>::Ok, ::bison::Error> =
                #extractor(req, #param)
                    .map_err(::bison::Error::from);

            ::bison::extract::Transform::transform(result)?
        });
    }

    return Ok(quote_spanned! { field.ty.span() =>
        let result: ::std::result::Result<<#ty as ::bison::extract::Transform<_>>::Ok, ::bison::Error> =
            ::bison::extract::default(req, ::bison::extract::arg::DefaultArgument::new(#field_name))
                .map_err(::bison::Error::from);

        ::bison::extract::Transform::transform(result)?
    });
}

fn bad_extractor(tokens: impl ToTokens) -> Error {
    Error::new_spanned(
        tokens,
        "expected #[cx(extractor)] or #[cx(extractor = ...)]",
    )
}

#[proc_macro_attribute]
pub fn async_trait_not_send_internal(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let out = quote! {
        #[::async_trait::async_trait(?Send)]
        #input
    };
    out.into()
}

#[proc_macro_attribute]
pub fn async_trait_not_send(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let out = quote! {
        #[::bison::async_trait(?Send)]
        #input
    };
    out.into()
}
