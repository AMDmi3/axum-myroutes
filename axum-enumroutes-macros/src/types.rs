// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::str::FromStr;

use crate::path::Path;

pub enum Method {
    Get,
    Post,
}

pub struct ParseMethodError;

impl FromStr for Method {
    type Err = ParseMethodError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "get" => Ok(Self::Get),
            "post" => Ok(Self::Post),
            _ => Err(ParseMethodError),
        }
    }
}

pub struct Route {
    pub method: Method,
    pub path: Path,
    pub handler: syn::Expr,
    pub props: Option<syn::Expr>,
}

pub struct Variant {
    pub ident: syn::Ident,
    pub route: Route,
    pub other_attributes: Vec<syn::Attribute>,
}

pub struct Enum {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub variants: Vec<Variant>,
    pub state_type: syn::Type,
    pub props_type: syn::Type,
    pub static_props: bool,
    pub default_props: bool,
}
