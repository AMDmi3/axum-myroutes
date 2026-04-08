// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use itertools::Itertools;
use proc_macro::TokenStream;
use syn::{ItemEnum, parse::ParseStream, spanned::Spanned};

use crate::path::Path;
use crate::types::{Enum, Route, Variant};

#[allow(clippy::large_enum_variant, reason = "should not matter much")]
enum AttributeKind {
    Route(Route),
    Other,
}

fn parse_variant_attribute(attr: &syn::Attribute) -> syn::Result<AttributeKind> {
    if let syn::Meta::List(list) = &attr.meta {
        let attr_name = list
            .path
            .segments
            .first()
            .map(|segment| segment.ident.to_string());
        if let Some(attr_name) = attr_name
            && let Ok(method) = attr_name.parse()
        {
            let mut path: Option<Path> = None;
            let mut handler: Option<syn::Expr> = None;
            let mut props: Option<syn::Expr> = None;

            attr.parse_args_with(|input: ParseStream| {
                path = Some(Path::parse(&input.parse::<syn::LitStr>().map_err(
                    |e| {
                        syn::Error::new(
                            e.span(),
                            "expected route path as the first argument".to_string(),
                        )
                    },
                )?)?);

                while input.peek(syn::Token![,]) {
                    input.parse::<syn::Token![,]>()?;
                    if input.is_empty() {
                        break;
                    }
                    let key: syn::Ident = input.parse()?;
                    input.parse::<syn::Token![=]>()?;
                    match key.to_string().as_str() {
                        "handler" => {
                            handler = Some(input.parse()?);
                        }
                        "props" => {
                            props = Some(input.parse()?);
                        }
                        _ => {
                            return Err(syn::Error::new(
                                key.span(),
                                format!("unknown argument `{key}`"),
                            ));
                        }
                    }
                }
                Ok(())
            })?;

            let path = path.expect("path always expected to be filled");

            let Some(handler) = handler else {
                return Err(syn::Error::new(attr.span(), "missing `handler` argument"));
            };

            return Ok(AttributeKind::Route(Route {
                method,
                path,
                handler,
                props,
            }));
        }
    }

    Ok(AttributeKind::Other)
}

fn parse_variant(variant: syn::Variant) -> syn::Result<Variant> {
    // sanity check
    if let Some(discr) = variant.discriminant {
        let span = discr
            .0
            .span()
            .join(discr.1.span())
            .unwrap_or(discr.1.span());
        return Err(syn::Error::new(
            span,
            "enum variant should not have a discriminant",
        ));
    }

    let span = variant.span(); // preserve as variant will be partially deconstructed

    // parse
    let mut route: Option<Route> = None;
    let mut other_attributes: Vec<syn::Attribute> = vec![];
    for attr in variant.attrs {
        match parse_variant_attribute(&attr)? {
            AttributeKind::Route(new_route) => {
                if route.replace(new_route).is_some() {
                    return Err(syn::Error::new(
                        attr.span(),
                        "multiple route attributes per variant are not allowed",
                    ));
                }
            }
            AttributeKind::Other => other_attributes.push(attr),
        }
    }

    let Some(route) = route else {
        return Err(syn::Error::new(
            span,
            "enum variant missing route attribute",
        ));
    };

    Ok(Variant {
        ident: variant.ident,
        route,
        other_attributes,
    })
}

struct RootAttributeArgs {
    pub state_type: syn::Type,
    pub props_type: syn::Type,
    pub static_props: bool,
    pub default_props: bool,
}

fn parse_root_attribute_args(attr: TokenStream) -> syn::Result<RootAttributeArgs> {
    let mut res = RootAttributeArgs {
        state_type: syn::parse_quote!(()),
        props_type: syn::parse_quote!(()),
        static_props: false,
        default_props: true,
    };

    syn::parse::Parser::parse2(
        |input: ParseStream| {
            while !input.is_empty() {
                let key: syn::Ident = input.parse()?;
                input.parse::<syn::Token![=]>()?;
                match key.to_string().as_str() {
                    "state_type" => {
                        res.state_type = input.parse()?;
                    }
                    "props_type" => {
                        res.props_type = input.parse()?;
                    }
                    "static_props" => {
                        let lit: syn::LitBool = input.parse()?;
                        res.static_props = lit.value;
                    }
                    "default_props" => {
                        let lit: syn::LitBool = input.parse()?;
                        res.default_props = lit.value;
                    }
                    _ => {
                        return Err(syn::Error::new(
                            key.span(),
                            format!("unknown argument `{key}`"),
                        ));
                    }
                }

                if !input.is_empty() {
                    input.parse::<syn::Token![,]>()?;
                }
            }
            Ok(())
        },
        attr.into(),
    )?;

    Ok(res)
}

pub fn parse_enum(attr: TokenStream, item: TokenStream) -> syn::Result<Enum> {
    let root_attribute_args = parse_root_attribute_args(attr)?;

    let r#enum = syn::parse::<ItemEnum>(item)?;

    // sanity check
    if let Some(param) = r#enum.generics.params.first() {
        return Err(syn::Error::new(param.span(), "enum should not be generic"));
    }

    let variants: Vec<_> = r#enum
        .variants
        .into_iter()
        .map(parse_variant)
        .try_collect()?;

    Ok(Enum {
        vis: r#enum.vis,
        ident: r#enum.ident,
        variants,
        state_type: root_attribute_args.state_type,
        props_type: root_attribute_args.props_type,
        static_props: root_attribute_args.static_props,
        default_props: root_attribute_args.default_props,
    })
}
