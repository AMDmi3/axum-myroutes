# axum-enumroutes

This crate provides convenient and reliable way to work with routes in `axum`. Define routes along with their methods, paths and handlers in an enum, add these to `axum::Router` with a single call, then refer to any route (and construct a link to it) via corresponding enum variant, which provides compile time checked internal links for your application. Also, axum extractor is provided for handlers to be aware of their routes, and capable of constructing links to themselves.

## Features

- Specify routes as enum, refer to routes via enum variants.
- Query route info, and construct path to a route, specifying/overriding path parameters, query parameters and fragment.
- Optionally specify additional route properties with a custom type.
- Extractors for axum handlers to get current route with filled parameters.

## Example

```rust
use axum_enumroutes::routes;

// Specify routes
#[derive(Clone, Copy)]
#[routes]
enum Route {
    #[get("/items.{id}", handler = item_by_id)]
    ItemById,
}

// Inspect routes and construct paths
assert_eq!(Route::ItemById.path(), "/items/{id}");
assert_eq!(Route::ItemById.name(), "ItemById");
// Parameter needs to be specified
assert!(Route::ItemById.url_for().build().is_err());
assert_eq!(Route::ItemById.url_for().param("id", 1).build().unwrap(), "/items/1".to_string());

// `Self``Route` is an extractor generated for `Route` enum
async fn item_by_id(route: SelfRoute) {
    assert_eq!(route.path(), "/items/{id}");
    assert_eq!(route.name(), "ItemById");
    // Current parameters are already filled...
    assert!(route.url_for().build().is_ok());
    // ...but may be overwritten
    assert_eq!(route.url_for().param("id", 2).build().unwrap(), "/items/2".to_string());
}

#[tokio::main]
async fn main() {
    // Add all routes from enum to an axum::Router
    let app = Route::to_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## Safety

This crate uses `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% safe Rust.

## Supported Rust versions

`axum-enumroutes` supports current stable Rust version and 2 most recent minor releases before it.
Increasing MSRV is not considered a semver breaking change as long as it follows this policy.
The current MSRV is 1.88.

## Documentation

See https://docs.rs/axum-enumroutes/latest/axum-enumroutes/ for complete documentation.

## Other implementations of the same concept

- [axum-routes](https://crates.io/crates/axum-routes)

## License

- MIT OR Apache-2.0
