// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use syn::{parse::ParseStream, spanned::Spanned};

use crate::path::Path;

use crate::method::Method;

pub struct Route {
    pub method: Method,
    pub path: Path,
    pub handler: syn::Expr,
    pub props: Option<syn::Expr>,
}

#[allow(clippy::large_enum_variant, reason = "should not matter much")]
pub enum AttributeKind {
    Route(Route),
    Other,
}

impl AttributeKind {
    pub fn parse(attr: &syn::Attribute) -> syn::Result<AttributeKind> {
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
}
