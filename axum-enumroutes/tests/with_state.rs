// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use axum::extract::State;
use axum_enumroutes::routes;

#[derive(Clone)]
struct AppState {}

async fn _handler(_: State<AppState>) {}

#[derive(Clone)]
#[routes(state_type = AppState)]
enum Route {
    #[get("/", handler = _handler)]
    Foo,
}

#[tokio::test]
async fn test_routes_and_extractors() {
    let _: axum::Router<()> = Route::to_router().with_state(AppState {});
}
