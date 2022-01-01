extern crate proc_macro;

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Peek};
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

    for param in &input.generics.params {
        match param {
            GenericParam::Type(_) => {
                return Err(Error::new_spanned(
                    &input.generics.params,
                    "generics type parameters are not supported",
                ))
            }
            GenericParam::Lifetime(_) => {
                return Err(Error::new_spanned(
                    &input.generics.params,
                    "lifetime parameters are not supported",
                ));
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

    let (_, _, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl ::bison::Context for #name #where_clause {
            type Future = ::bison::bounded::BoxFuture<'static, Result<Self, ::bison::Rejection>>;

            fn extract(req: ::bison::Request) -> Self::Future {
                Box::pin(async move { Ok(#name { #(#fields)* }) })
            }
        }
    })
}

fn extract(field: &Field) -> Result<TokenStream> {
    let field_name = field.ident.as_ref().unwrap().to_string();
    let ty = &field.ty;

    for attr in &field.attrs {
        if attr.path.get_ident().map(Ident::to_string).as_deref() != Some("cx") {
            continue;
        }

        let MyMeta { extractor, arg } = attr.parse_args()?;

        let arg = match arg {
            Some(arg) => {
                quote_spanned! { ty.span() => ::bison::extract::arg::Argument::new(#field_name, #arg) }
            }
            None => {
                quote_spanned! { ty.span() => ::bison::extract::arg::DefaultArgument::new(#field_name) }
            }
        };

        return Ok(quote_spanned! { ty.span() =>
            let result: ::std::result::Result<<#ty as ::bison::extract::Transform<_>>::Ok, ::bison::Rejection> =
                #extractor(&req, #arg)
                    .await
                    .map_err(::bison::Rejection::from);

            ::bison::extract::Transform::transform(result)?
        });
    }

    return Ok(quote_spanned! { field.ty.span() =>
        let result: ::std::result::Result<<#ty as ::bison::extract::Transform<_>>::Ok, ::bison::Rejection> =
            ::bison::extract::default(&req, ::bison::extract::arg::DefaultArgument::new(#field_name))
                .await
                .map_err(::bison::Rejection::from);

        ::bison::extract::Transform::transform(result)?
    });
}

struct MyMeta {
    extractor: Expr,
    arg: Option<Expr>,
}

impl Parse for MyMeta {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let extractor: Expr = syn::parse2(parse_until(input, Token![=])?)?;

        let arg = if input.is_empty() {
            None
        } else {
            let _: Token![=] = input.parse()?;
            Some(input.parse::<Expr>()?)
        };

        Ok(MyMeta { extractor, arg })
    }
}

fn parse_until<E: Peek>(input: ParseStream, end: E) -> Result<TokenStream> {
    let mut tokens = TokenStream::new();
    while !input.is_empty() && !input.peek(end) {
        let next: TokenTree = input.parse()?;
        tokens.extend(Some(next));
    }
    Ok(tokens)
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
