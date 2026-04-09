// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use axum_myroutes::routes;

async fn _handler() {}

#[derive(Clone)]
#[routes]
enum Route {
    #[get("/foo", handler = _handler)]
    Foo,
    #[cfg(true)]
    #[get("/bar", handler = _handler)]
    Bar,
    #[cfg(false)]
    #[get("/bar", handler = _handler)]
    Baz,
}

fn main() {}
