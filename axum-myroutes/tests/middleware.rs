// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use axum::extract::Request;
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum_myroutes::routes;

async fn _handler() {}

#[derive(Clone)]
#[routes]
enum Route {
    #[get("/", handler = _handler)]
    Foo,
}

async fn middleware(route: MyRoute, request: Request, next: Next) -> impl IntoResponse {
    assert_eq!(route.name(), "Foo");
    next.run(request).await
}

#[tokio::test]
async fn test_middleware() {
    let router =
        Route::to_router_with(|router| router.route_layer(middleware::from_fn(middleware)));
    let test_server = axum_test::TestServer::new(router);
    let _ = test_server.get("/").await;
}
