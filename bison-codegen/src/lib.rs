extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::*;

const REQUEST_LIFETIME: &str = "'bison_request";

#[proc_macro_derive(Context, attributes(cx))]
pub fn derive_context(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand(input: DeriveInput) -> Result<TokenStream> {
    let request_lifetime = Lifetime::new(REQUEST_LIFETIME, Span::call_site());

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
                #name: #extracted,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let mut with_req_lifetime = input.generics.clone();
    with_req_lifetime
        .params
        .push(GenericParam::Lifetime(LifetimeDef {
            attrs: Vec::new(),
            lifetime: request_lifetime.clone(),
            colon_token: None,
            bounds: Default::default(),
        }));

    let (impl_generics, _, _) = with_req_lifetime.split_for_impl();
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();

    let context_ty_generics = ty_lifetime
        .as_ref()
        .map(|_| quote! { <#request_lifetime> })
        .unwrap_or_default();

    let with_context = quote! {
        impl #impl_generics ::bison::WithContext<#request_lifetime> for #name #ty_generics #where_clause {
            type Context = #name #context_ty_generics;
        }
    };

    match with_req_lifetime.params.last_mut().unwrap() {
        GenericParam::Lifetime(l) => {
            l.bounds = ty_lifetime.into_iter().collect();
        }
        _ => unreachable!(),
    }

    let (impl_generics, _, _) = with_req_lifetime.split_for_impl();
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();

    let context = quote! {
        impl #impl_generics ::bison::Context<#request_lifetime> for #name  #ty_generics #where_clause {
            fn extract(
                req: &#request_lifetime ::bison::http::Request
            ) -> ::std::pin::Pin<::std::boxed::Box<
                    dyn ::bison::bounded::Future<Output = Result<Self, ::bison::Error>> + #request_lifetime>>
            {
                Box::pin(async move { Ok(#name { #(#fields)* }) })
            }
        }
    };

    Ok(quote! {
        #with_context
        #context
    })
}

fn extract(field: &Field) -> Result<TokenStream> {
    let field_name = field.ident.as_ref().unwrap().to_string();

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
            Some(param) => quote! { Param::new(#field_name, #param) },
            None => quote! { NoParam::new(#field_name) },
        };

        return Ok(quote_spanned! { field.ty.span() =>
            #extractor(req, ::bison::extract::#param).map_err(::bison::Error::from)?
        });
    }

    return Ok(quote_spanned! { field.ty.span() =>
        ::bison::extract::default(req, ::bison::extract::NoParam::new(#field_name))
            .map_err(::bison::Error::from)?
    });
}

fn bad_extractor(tokens: impl ToTokens) -> Error {
    Error::new_spanned(
        tokens,
        "expected #[cx(extractor)] or #[cx(extractor = ...)]",
    )
}
