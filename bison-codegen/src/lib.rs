extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Error, Field, Fields, Lit, Meta,
    NestedMeta, Result,
};

#[proc_macro_derive(HasContext, attributes(param, header))]
pub fn derive_has_context(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand(input: &DeriveInput) -> Result<TokenStream> {
    let strukt = match &input.data {
        Data::Struct(s) => s,
        Data::Enum(_) => return Err(Error::new_spanned(input, "expected struct, found enum")),
        Data::Union(_) => return Err(Error::new_spanned(input, "expected struct, found union")),
    };

    let name = &input.ident;

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

    let fields = fields
        .named
        .iter()
        .map(|f| {
            let name = &f.ident.as_ref().unwrap();
            let extracted = extract_field(f)?;
            Ok(quote! {
                #name: { #extracted },
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let out = quote! { impl<__State: ::bison::State> ::bison::HasContext<__State> for #name {
            type ConstructionError = ::bison::Error;
            type ConstructionFuture = ::bison::send::BoxFuture<'static, ::std::result::Result<Self, ::bison::Error>>;

            fn extract(request: ::bison::Request<__State>) -> Self::ConstructionFuture {
                Box::pin(async move {
                    Ok(#name { #(#fields)* })
                })
            }
        }
    };

    Ok(out)
}

fn extract_field(field: &Field) -> Result<TokenStream> {
    for attr in &field.attrs {
        let attr = attr.parse_meta()?;

        let path = match &attr {
            Meta::Path(path) => path,
            Meta::List(list) => &list.path,
            Meta::NameValue(nv) => &nv.path,
        };

        if let Some(extractor) = path.get_ident().map(ToString::to_string) {
            match extractor.as_str() {
                PARAM_EXTRACTOR => {
                    return extract_param(&field, &attr);
                }
                HEADER_EXTRACTOR => {
                    return extract_header(&field, &attr);
                }
                _ => {}
            }
        }
    }

    return Err(Error::new_spanned(field, EXPECTED_EXTRACTORS));
}

const PARAM_EXTRACTOR: &'static str = "param";
const HEADER_EXTRACTOR: &'static str = "header";
const EXPECTED_EXTRACTORS: &'static str = "expected #[param], #[data], or #[header] attribute";

fn single_arg_or_field_name(
    field: &Field,
    meta: &Meta,
    multiple_params_err: &str,
    not_string_err: &str,
    key_value_err: &str,
) -> Result<String> {
    let name = match meta {
        Meta::Path(_) => None,
        Meta::List(meta) => {
            if meta.nested.len() > 1 {
                return Err(Error::new_spanned(meta, multiple_params_err));
            }

            match meta.nested.first() {
                Some(NestedMeta::Lit(Lit::Str(name))) => Some(name.value()),
                Some(other) => {
                    return Err(Error::new_spanned(other, not_string_err));
                }
                None => None,
            }
        }
        Meta::NameValue(meta) => {
            return Err(Error::new_spanned(meta, key_value_err));
        }
    };

    Ok(name.unwrap_or_else(|| field.ident.as_ref().unwrap().to_string()))
}

fn extract_param(field: &Field, meta: &Meta) -> Result<TokenStream> {
    let name = single_arg_or_field_name(
        field,
        meta,
        "expected single argument for parameter name",
        "expected string as parameter name",
        "expected format: '#[param(\"name\")]'",
    )?;

    let ty_span = field.ty.span();
    Ok(quote_spanned! { ty_span =>
        let param = request.param(#name).ok_or(::bison::error::ParamNotFound::new())?;
        ::bison::context::ParamContext::extract(param)?
    })
}

fn extract_header(field: &Field, meta: &Meta) -> Result<TokenStream> {
    let name = single_arg_or_field_name(
        field,
        meta,
        "expected single argument for header name",
        "expected string as header name",
        "expected format: '#[header(\"name\")]'",
    )?;

    let ty_span = field.ty.span();
    Ok(quote_spanned! { ty_span =>
        let values = request.headers().get_all(#name);
        if values.iter().count() == 0 {
            return Err(::bison::error::ParamNotFound::new().into());
        }

        ::bison::context::HeaderContext::extract(values)?
    })
}

// #[derive(Extract)]
// struct Foo<'r> {
//     #[extract(param)]
//     id: usize,
//     #[extract(header = "BEARER")]
//     token: &'r str,
//     #[extract(state)]
//     db: &'r Database
// }
// 
// #[derive(Extract)]
// struct Foo<'r> {
//     #[param]
//     id: usize,
//     #[header("BEARER")]
//     token: &'r str,
//     #[state]
//     db: &'r Database
// }
// 
// #[derive(Extract)]
// struct Foo<'r> {
//     #[param]
//     id: usize,
//     #[header(name = "BEARER")]
//     token: &'r str,
//     #[state]
//     db: &'r Database
// }
// 
// #[derive(Context)]
// struct Foo<'r> {
//     #[cx(param)]
//     id: usize,
//     #[cx(header = "BEARER")]
//     token: &'r str
//     #[cx(state)]
//     db: &'r Database
// }
// 
// #[derive(Context)]
// struct Foo<'r> {
//     #[ctx(param)]
//     id: usize,
//     #[ctx(header = "BEARER")]
//     token: &'r str,
//     #[ctx(state)]
//     db: &'r Database
// }
// 
// #[derive(Extract)]
// struct Foo<'r> {
//     #[from(param)]
//     id: usize,
//     #[from(header = "BEARER")]
//     token: &'r str,
//     #[from(state)]
//     db: &'r Database
// }
