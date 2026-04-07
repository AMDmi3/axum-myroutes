// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg(feature = "nightly")]
#![feature(const_default)]
#![feature(const_trait_impl)]
#![feature(derive_const)]

use axum_enumroutes::routes;

#[derive_const(Default)]
struct Props {
    someprop: bool,
}

async fn _handler() -> &'static str {
    ""
}

#[derive(Clone, Copy)]
#[routes(props_type = Props, static_props = true, default_props = true)]
enum Routes {
    #[get("/foo/{id}", handler = _handler)] // const Props::default is used
    Foo,
    #[get("/bar/{id}", handler = _handler, props = Props { someprop: true })]
    Bar,
}

#[test]
fn test_props() {
    assert_eq!(Routes::Foo.path(), "/foo/{id}");
    assert_eq!(Routes::Foo.name(), "Foo");
    assert_eq!(Routes::Foo.props().someprop, false);
    assert_eq!(Routes::Bar.path(), "/bar/{id}");
    assert_eq!(Routes::Bar.name(), "Bar");
    assert_eq!(Routes::Bar.props().someprop, true);
}
