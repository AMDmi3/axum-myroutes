// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use axum_enumroutes::routes;

async fn _handler() -> &'static str {
    ""
}

#[derive(Clone)]
#[routes]
enum Route {
    #[get("/", handler = _handler)]
    Index(u64),
}

fn main() {}
