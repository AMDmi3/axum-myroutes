// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![forbid(unsafe_code)]

mod generate;

// Each module is dedicated to parsing one bit of enum definition:
//
// #[routes] ← routes_attribute                          ⎫
// enum Routes {                                         ⎪
//     method                                            ⎪
//        │  path                                        ⎬ enum
//        │   │                                          ⎪
//     #[get("/", handler=)] ← variant_attribute ⎱ vari  ⎪
//     Home                                      ⎰  ant  ⎪
// }                                                     ⎭
mod r#enum;
mod method;
mod path;
mod routes_attribute;
mod variant;
mod variant_attribute;

use proc_macro::TokenStream;

use crate::r#enum::Enum;
use crate::generate::generate;

#[proc_macro_attribute]
pub fn routes(attr: TokenStream, item: TokenStream) -> TokenStream {
    let r#enum = match Enum::parse(attr, item) {
        Ok(r#enum) => r#enum,
        Err(err) => return err.into_compile_error().into(),
    };

    match generate(r#enum) {
        Ok(expanded) => expanded,
        Err(err) => err.into_compile_error().into(),
    }
}
