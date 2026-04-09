// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use axum_myroutes::routes;

struct Props{}

async fn _handler() -> &'static str {
    ""
}

#[derive(Clone)]
#[routes]
enum RouteMinimal {
    #[get("/", handler = _handler)]
    Index,
}

// trailing commas
#[derive(Clone)]
#[routes(static_props = false,)]
enum RouteTrailingCommas {
    #[get("/", handler = _handler,)]
    Index,
}

// all attrs
#[derive(Clone)]
#[routes(props_type = Props, static_props = false, default_props = false)]
enum RouteAllAttrs {
    #[get("/", handler = _handler, props = Props{})]
    Index,
    #[post("/", handler = _handler, props = Props{})]
    Post,
}

fn main() {}
