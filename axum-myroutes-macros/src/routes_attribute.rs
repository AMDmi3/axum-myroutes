// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use proc_macro::TokenStream;
use syn::parse::ParseStream;

pub struct RoutesAttributeArgs {
    pub state_type: syn::Type,
    pub props_type: syn::Type,
    pub static_props: bool,
    pub default_props: bool,
}

impl RoutesAttributeArgs {
    pub fn parse(attr: TokenStream) -> syn::Result<RoutesAttributeArgs> {
        let mut res = RoutesAttributeArgs {
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
}
