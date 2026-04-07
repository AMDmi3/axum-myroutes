// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use axum_enumroutes::routes;

#[derive(Clone)]
struct State {}

async fn _handler() {}

#[derive(Clone)]
#[routes]
enum Route {
    #[get("/", handler = _handler)]
    Foo,
}

#[tokio::test]
async fn test_routes_and_extractors() {
    let _: axum::Router<()> = Route::add_to_router(axum::Router::new()).with_state(State {});
    //let _: axum::Router<State> = Route::add_to_router(axum::Router::new()).with_state(State{});
}
