// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use itertools::Itertools;
use proc_macro::TokenStream;
use syn::{ItemEnum, spanned::Spanned};

use crate::routes_attribute::RootAttributeArgs;
use crate::variant::Variant;

pub struct Enum {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub variants: Vec<Variant>,
    pub state_type: syn::Type,
    pub props_type: syn::Type,
    pub static_props: bool,
    pub default_props: bool,
}

impl Enum {
    pub fn parse(attr: TokenStream, item: TokenStream) -> syn::Result<Enum> {
        let root_attribute_args = RootAttributeArgs::parse(attr)?;

        let r#enum = syn::parse::<ItemEnum>(item)?;

        // sanity check
        if let Some(param) = r#enum.generics.params.first() {
            return Err(syn::Error::new(param.span(), "enum should not be generic"));
        }

        let variants: Vec<_> = r#enum
            .variants
            .into_iter()
            .map(Variant::parse)
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
}
