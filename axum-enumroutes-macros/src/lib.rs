// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod generate;
mod parse;
mod types;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn routes(attr: TokenStream, item: TokenStream) -> TokenStream {
    let r#enum = match parse::parse_enum(attr, item) {
        Ok(r#enum) => r#enum,
        Err(err) => return err.into_compile_error().into(),
    };

    match generate::generate(r#enum) {
        Ok(expanded) => expanded,
        Err(err) => err.into_compile_error().into(),
    }
}
