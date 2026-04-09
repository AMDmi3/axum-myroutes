// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use syn::spanned::Spanned;

use crate::variant_attribute::{Route, VariantAttribute};

pub struct Variant {
    pub ident: syn::Ident,
    pub route: Route,
    pub other_attributes: Vec<syn::Attribute>,
}

impl Variant {
    pub fn parse(variant: syn::Variant) -> syn::Result<Variant> {
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
            match VariantAttribute::parse(&attr)? {
                VariantAttribute::Route(new_route) => {
                    if route.replace(new_route).is_some() {
                        return Err(syn::Error::new(
                            attr.span(),
                            "multiple route attributes per variant are not allowed",
                        ));
                    }
                }
                VariantAttribute::Other => other_attributes.push(attr),
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
}
