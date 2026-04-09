// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use axum_myroutes::routes;

struct Props {
    someprop: u64,
}

async fn _handler() -> &'static str {
    ""
}

#[derive(Clone, Copy)]
#[routes(props_type = Props, static_props = true, default_props = false)]
enum Routes {
    #[get("/foo/{id}", handler = _handler, props = Props { someprop: 0 })]
    Foo,
    #[get("/bar/{id}", handler = _handler, props = Props { someprop: 1 })]
    Bar,
}

#[test]
fn test_props() {
    assert_eq!(Routes::Foo.path(), "/foo/{id}");
    assert_eq!(Routes::Foo.name(), "Foo");
    assert_eq!(Routes::Foo.props().someprop, 0);
    assert_eq!(Routes::Bar.path(), "/bar/{id}");
    assert_eq!(Routes::Bar.name(), "Bar");
    assert_eq!(Routes::Bar.props().someprop, 1);
}
