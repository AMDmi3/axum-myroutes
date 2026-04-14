// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use axum::extract::Path;
use axum_myroutes::routes;

#[derive(Default)]
struct Props {
    pub someprop: bool,
}

#[derive(Clone)]
#[routes(props_type = Props)]
enum Route {
    #[allow(unused)] // check if other attributes are accepted
    #[get("/foo/{id}", handler = foo)]
    Foo,
    #[get("/bar/{id}", handler = bar, props = Props{someprop: true})]
    Bar,
}

async fn foo(route: MyRoute, Path(id): Path<u64>) -> String {
    format!(
        "handler=foo, base={}, route={}, someprop={}, to_self={}, to_next={}, to_other={}",
        route.base().name(),
        route.name(),
        route.props().someprop,
        route.url_for().build().unwrap(),
        route
            .url_for()
            .path_param("id", id + 1)
            .unwrap()
            .build()
            .unwrap(),
        Route::Bar
            .url_for()
            .path_param("id", id)
            .unwrap()
            .build()
            .unwrap(),
    )
}

async fn bar(route: MyRoute, Path(id): Path<u64>) -> String {
    format!(
        "handler=bar, base={}, route={}, someprop={}, to_self={}, to_next={}, to_other={}",
        route.base().name(),
        route.name(),
        route.props().someprop,
        route.url_for().build().unwrap(),
        route
            .url_for()
            .path_param("id", id + 1)
            .unwrap()
            .build()
            .unwrap(),
        Route::Foo
            .url_for()
            .path_param("id", id)
            .unwrap()
            .build()
            .unwrap(),
    )
}

#[tokio::test]
async fn test_routes_and_extractors() {
    let router = Route::to_router();
    let test_server = axum_test::TestServer::new(router);

    let response = test_server.get("/foo/1").await;
    response.assert_status_ok();
    response.assert_text(
        "handler=foo, base=Foo, route=Foo, someprop=false, to_self=/foo/1, to_next=/foo/2, to_other=/bar/1",
    );

    let response = test_server.get("/bar/2").await;
    response.assert_status_ok();
    response.assert_text(
        "handler=bar, base=Bar, route=Bar, someprop=true, to_self=/bar/2, to_next=/bar/3, to_other=/foo/2",
    );
}
