// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use axum::extract::Request;
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum_enumroutes::routes;

async fn _handler() {}

#[derive(Clone)]
#[routes]
enum Route {
    #[get("/", handler = _handler)]
    Foo,
}

async fn middleware(request: Request, next: Next) -> impl IntoResponse {
    // we actually need is_some here
    assert!(!request.extensions().get::<Route>().is_some());
    let response = next.run(request).await;
    assert!(response.extensions().get::<Route>().is_some());
    response
}

#[tokio::test]
async fn test_middleware() {
    let router =
        Route::add_to_router(axum::Router::new()).route_layer(middleware::from_fn(middleware));
    let test_server = axum_test::TestServer::new(router);
    let _ = test_server.get("/").await;
}
