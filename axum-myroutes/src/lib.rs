// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This crate provides convenient and reliable way to work with routes
//! in [`axum`]. Define routes along with their methods, paths and handlers
//! in an enum, add construct [`axum::Router`] with a single call, then refer
//! to any route (and construct a link to it) via corresponding enum variant,
//! which provides compile time checked internal links for your application.
//! Also, axum extractor is provided for handlers to be aware of their routes,
//! and capable of constructing links to themselves.
//!
//! # Example
//!
//! ```no_run
//! use axum::extract::Path;
//! use axum_myroutes::routes;
//!
//! // Specify routes
//! #[derive(Clone, Copy)]
//! #[routes]
//! enum Route {
//!     #[get("/", handler = home)]
//!     Home,
//!     #[get("/items/{id}", handler = item_by_id)]
//!     ItemById,
//! }
//!
//! async fn home() -> String {
//!     // Construct links to routes
//!     format!(
//!         "<a href={}>To first item</a>",
//!         Route::ItemById.url_for().param("id", 1).build().unwrap()
//!     )
//! }
//!
//! // My{...} extractor is generated for the enum
//! async fn item_by_id(route: MyRoute, Path(id): Path<u64>) -> String {
//!     format!(
//!         "<a href={}>To home</a><a href={}>To self</a><a href={}>To next</a>",
//!         Route::Home.url_for().build().unwrap(),
//!         // Construct links to current route, parameters are already filled...
//!         route.url_for().build().unwrap(),
//!         // ...but can be modified
//!         route.url_for().param("id", id + 1).build().unwrap(),
//!     )
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Route::to_router();
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```
//!
//! ## Route methods and path construction
//!
//! `#[routes]` enum has methods to query information related to route and to construct
//! paths to it. It's possible to specify path params with `param()`, query params with
//! `query_param()` and fragment with `fragment()` methods of [`PathBuilder`] returned by
//! `url_for()`.
//!
//! ```
//! # use axum_myroutes::routes;
//! # async fn handler(){}
//! # #[derive(Clone, Copy)]
//! # #[routes]
//! # enum Route {
//! #     #[get("/{id}", handler = handler)]
//! #     ItemById,
//! # }
//! #
//! // Get route path (pattern)
//! assert_eq!(Route::ItemById.path(), "/{id}");
//!
//! // Get route name (name of enum variant)
//! assert_eq!(Route::ItemById.name(), "ItemById");
//!
//! // Construct url
//! assert_eq!(Route::ItemById.url_for().param("id", 123).build().unwrap(), "/123");
//!
//! // Error on missing parameter
//! assert!(Route::ItemById.url_for().build().is_err());
//!
//! // Can also set query params and fragment
//! assert_eq!(
//!     Route::ItemById
//!         .url_for()
//!         .param("id", 123)
//!         .query_param("foo", "bar")
//!         .fragment("frag")
//!         .build()
//!         .unwrap(),
//!     "/123?foo=bar#frag"
//! );
//! ```
//!
//! ## Route props
//!
//! A type to store additional route properties can be provided, set per-route, and
//! retrieved with route method.
//!
//! There are options to toggle [`Default`] requirement on props type, and to allow
//! static construction of props, enabled through [`routes`] arguments.
//!
//! ```
//! # use axum_myroutes::routes;
//! # async fn handler(){}
//! #[derive(Default)]
//! struct RouteProps {
//!     require_auth: bool,
//! }
//!
//! #[derive(Clone, Copy)]
//! #[routes(props_type = RouteProps)]
//! enum Route {
//!     #[get("/public", handler = handler)]
//!     Public,
//!     #[get("/private", handler = handler, props = RouteProps { require_auth: true })]
//!     Private,
//! }
//!
//! assert_eq!(Route::Public.props().require_auth, false);
//! assert_eq!(Route::Private.props().require_auth, true);
//! ```
//!
//! ## Extractors
//!
//! Extractor type is automatically provided for the route enum, with the same
//! name prefixed with `My`.
//!
//! ```
//! # use axum_myroutes::routes;
//! # #[derive(Default)]
//! # struct RouteProps {
//! #     require_auth: bool,
//! # }
//! # #[derive(Clone, Copy)]
//! # #[routes(props_type = RouteProps)]
//! # enum Route {
//! #     #[get("/{id}", handler = item_by_id)]
//! #     ItemById,
//! # }
//! async fn item_by_id(route: MyRoute) {
//!     // Same methods as route variant
//!     assert_eq!(route.path(), "/");
//!     assert_eq!(route.name(), "ItemById");
//!     assert_eq!(route.props().require_auth, false);
//!
//!     // In extractor, parameters are already filled,
//!     // so path to self can be constructed right away
//!     assert!(route.url_for().build().is_ok());
//! }
//! ```
//!
//! ## Router with state
//!
//! If router with state is used (e.g. `.with_state()` is called on a router), the
//! state type must be passed to `#[routes]` argument:
//!
//! ```no_run
//! # use axum_myroutes::routes;
//! #[derive(Clone)]
//! struct AppState;
//!
//! # async fn handler(_: axum::extract::State<AppState>) {}
//! #[derive(Clone, Copy)]
//! #[routes(state_type = AppState)]  // note state_type argument
//! enum Route {
//!     #[get("/", handler = handler)]
//!     Home,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Route::to_router()
//!         .with_state(AppState);  // add state as usual
//!     # let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//!     # axum::serve(listener, app).await.unwrap();
//! }
//! ```
//!
//! ## Middleware with access to current route
//!
//! Current route can be accessed from a middleware, and this provides a powerful
//! mechanism to control routes behavior in a centralized way with route props.
//! However, for a middleware to see a current route, it should be added to a router
//! _before_ the route information is added, which means you can't add such middleware
//! to constructed `Router`. Instead, use `to_router_with()` to intervene with the
//! router construction and insert middleware a the right spot.
//!
//! ```no_run
//! use axum::{extract::Request, middleware::{from_fn, Next}, response::IntoResponse};
//! # use axum_myroutes::routes;
//! # #[derive(Default)]
//! # struct RouteProps { require_auth: bool }
//! # async fn handler() {}
//! # #[derive(Clone)]
//! # #[routes(props_type = RouteProps)]
//! # enum Route {
//! #     #[get("/", handler = handler)]
//! #     Home,
//! # }
//! async fn middleware(route: MyRoute, request: Request, next: Next) -> impl IntoResponse {
//!     if route.props().require_auth {
//!         // check auth
//!     }
//!     next.run(request).await
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Route::to_router_with(|route| route.layer(from_fn(middleware)));
//!     # let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//!     # axum::serve(listener, app).await.unwrap();
//! }
//! ```

#![forbid(unsafe_code)]

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use indexmap::IndexMap;

/// Main attribute macro for routes enum.
///
/// Use this on routes enum. Each enum variant should also be marked
/// with an attribute defining a HTTP method and path to handle, and
/// the handler.
///
/// # Example
///
/// ```
/// use axum_myroutes::routes;
///
/// async fn handler() {}
///
/// #[derive(Clone, Copy)]
/// #[routes]
/// enum Route {
///     #[get("/", handler = handler)]
///     Home,
/// }
/// ```
///
/// # Requirements
///
/// - Enum must be non-generic and cloneable (since the enum is trivial,
///   you may want to derive [`Copy`] as well).
/// - Variants must not contain any data fields or a discriminant,
///   each variant must have a single route attribute.
/// - Route paths must follow modern [`axum`] placeholder syntax,
///   e.g. `/{foo}/{bar}` and not `/:foo/:bar`.
///
/// # Macro attribute arguments
///
/// - `state_type` (default is unit type `()`) - type for route state. If you
///   call `.with_state()` on the router, it should be the same type.
/// - `props_type` (default is unit type `()`) - type for route properties.
///   When defined, you can set properties for each route with `props` parameter
///   on its route attribute.
/// - `static_props` (default false) - whether route properties are statically
///   constructed (otherwise they are constructed on each access). This matters
///   if properties contain heavy data, but require `const` construction.
/// - `default_props` (default true) - whether route properties have default value.
///   If true, this requires `Default` trait to be implemented on the props type,
///   in which case you may
///
/// Note that `static_props = true, default_props = true` case currently requires
/// rust nightly and `nightly` feature enabled for this crate, as it depends on a
/// bunch of unstable features.
///
/// # Route attributes
///
/// Allowed attributes corresponding to HTTP methods (or, rather,
/// [`axum::routing::method_routing::MethodRouter`] constructors): `any`, `connect`,
/// `delete`, `get`, `head`, `options`, `patch`, `post`, `put`, `trace`.
///
/// First argument is a string with route path, other arguments are key-value pairs:
///
/// - `handler` (required) - route handler for axum.
/// - `props` (required if `default_props` is false) - initialization expression
///   for route properties.
///
/// # Generated code
///
/// The macro generates enum with the same variants and the following methods:
///
/// - `path() -> &'static str` - returns route path, as specified in the attribute.
/// - `name() -> &'static str` - returns route enum variant name.
/// - `props() -> <props_type>` (`static_props` is false) or
///   `props() -> &'static <props_type>` (`static_props` is trie) - returns properties
///   defined for a route.
/// - `url_for() -> PathBuilder` - returns a path constructor for this route.
/// - `to_router() -> axum::Router` - creates [`axum::Router`] from the routes defined
///   in the enum.
/// - `to_router_with(f: Fn(axum::Router) -> axum::Router) -> axum::Router` - create
///   [`axum::Router`] with custization (such as adding middleware). This is required
///   to ensure layer ordering, as layers which use route extractor must be registered
///   before a layer which adds corresponding extension. In other words, to add middleware
///   with access to route extractor, you must add it through this method, and not on
///   an already constructed router.
///
/// Additionally, extractor type is generated, named with `My` prefix (e.g.
/// `MyRoute` for `enum Route`), with methods similar to that of an enum:
///
/// - `base() -> <enum type>` - returns base route enum variant.
/// - `path() -> &'static str` - returns route path, as specified in the attribute.
/// - `name() -> &'static str` - returns route enum variant name.
/// - `props() -> <props_type>` (`static_props` is false) or
///   `props() -> &'static <props_type>` (`static_props` is trie) - returns properties
///   defined for a route.
/// - `url_for_self() -> &PathBuilder` - returns a reference to path constructor for
///   this route with all parameters filled from the request, capable of building a
///   path to self.
/// - `url_for() -> PathBuilder` - returns a clone of path constructor for this route
///   with all parameters filled from the request, capable of building a path to self
///   and modifications of it (by adding, updating, or clearing any parameters).
pub use axum_myroutes_macros::routes;

#[doc(hidden)]
pub mod __private {
    pub use axum;

    pub enum PathSegment {
        Static(&'static str),
        Param(&'static str),
        CatchAllParam(&'static str),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PathConstructionError {
    #[error("missing path parameter {0}")]
    MissingPathParam(&'static str),
}

/// Route path builder.
///
/// This allows to fill a route path pattern with given parameter
/// values and construct a complete path.
#[derive(Clone)]
pub struct PathBuilder {
    path_segments: &'static [__private::PathSegment],
    path_params: HashMap<String, String>,
    query_params: IndexMap<String, String>,
    fragment: Option<String>,
}

impl PathBuilder {
    #[doc(hidden)]
    pub fn new(path_segments: &'static [__private::PathSegment]) -> Self {
        Self {
            path_segments,
            path_params: Default::default(),
            query_params: Default::default(),
            fragment: None,
        }
    }

    /// Adds or updates path parameter.
    pub fn param<K, V>(mut self, k: K, v: V) -> Self
    where
        K: Into<String>,
        V: std::fmt::Display,
    {
        self.path_params.insert(k.into(), format!("{}", v));
        self
    }

    /// Clears a path parameter.
    pub fn cleared_param<Q>(mut self, k: &Q) -> Self
    where
        String: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.path_params.remove(k.borrow());
        self
    }

    /// Adds or updates query parameter.
    pub fn query_param<K, V>(mut self, k: K, v: V) -> Self
    where
        K: Into<String>,
        V: std::fmt::Display,
    {
        self.query_params.insert(k.into(), format!("{}", v));
        self
    }

    /// Clears a path parameter.
    pub fn cleared_query_param<Q>(mut self, k: &Q) -> Self
    where
        String: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.query_params.shift_remove(k.borrow());
        self
    }

    /// Adds or updates URL fragment.
    pub fn fragment<S>(mut self, fragment: S) -> Self
    where
        S: std::fmt::Display,
    {
        self.fragment = Some(format!("{}", fragment));
        self
    }

    /// Clears URL fragment.
    pub fn cleared_fragment(mut self) -> Self {
        self.fragment = None;
        self
    }

    /// Fills parameters and fragment from another instance of `PathBuilder`.
    pub fn filled_from(mut self, other: &Self) -> Self {
        self.path_params = other.path_params.clone();
        self.query_params = other.query_params.clone();
        self.fragment = other.fragment.clone();
        self
    }

    /// Builds a path.
    ///
    /// Fills a path pattern with provided parameters, adding query parameters
    /// and fragment if nevessary, returning complete path which may be used
    /// as an internal link.
    ///
    /// All provided values are escaped with `url_escape::encode_component`,
    /// with the exception of catch-all parameters (`/{*rest}`) which are
    /// escaped with `url_escape::encode_path`.
    ///
    /// Fails if corresponding path pattern is invalid, or if required path
    /// parameter was not provided.
    pub fn build(&self) -> Result<String, PathConstructionError> {
        let mut res = String::new();
        for segment in self.path_segments {
            use __private::PathSegment;
            match segment {
                PathSegment::Static(text) => {
                    res += text;
                }
                PathSegment::Param(key) => {
                    if let Some(value) = self.path_params.get(*key) {
                        res += &url_escape::encode_component(value);
                    } else {
                        return Err(PathConstructionError::MissingPathParam(key));
                    }
                }
                PathSegment::CatchAllParam(key) => {
                    if let Some(value) = self.path_params.get(*key) {
                        res += &url_escape::encode_path(value);
                    } else {
                        return Err(PathConstructionError::MissingPathParam(key));
                    }
                }
            }
        }

        // TODO: may track unused path parameters and check for these here

        let mut first_query_param = true;
        for (key, value) in &self.query_params {
            res += if first_query_param { "?" } else { "&" };
            first_query_param = false;
            res += &url_escape::encode_component(&key);
            res += "=";
            res += &url_escape::encode_component(&value);
        }

        if let Some(fragment) = &self.fragment {
            res += "#";
            res += &url_escape::encode_component(&fragment);
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use __private::PathSegment;

    use super::*;

    #[test]
    fn test_basic_build() {
        static SEGMENTS: &[PathSegment] = &[
            PathSegment::Static("/static/{}/a"),
            PathSegment::Param("foo"),
            PathSegment::Static("/"),
            PathSegment::Param("bar"),
            PathSegment::Static("b/c"),
            PathSegment::CatchAllParam("baz"),
            PathSegment::Static("c"),
        ];
        let path = PathBuilder::new(SEGMENTS)
            .param("foo", "1")
            .param("bar", 2)
            .param("baz", String::from("3"))
            .query_param("aaa", 1)
            .query_param("bbb", 2)
            .fragment("frag")
            .build()
            .unwrap();
        assert_eq!(path, "/static/{}/a1/2b/c3c?aaa=1&bbb=2#frag");
    }

    #[test]
    fn test_escaping_param() {
        static SEGMENTS: &[PathSegment] = &[PathSegment::Param("a")];
        assert_eq!(
            PathBuilder::new(SEGMENTS)
                .param("a", "/%20#")
                .build()
                .unwrap(),
            "%2F%2520%23"
        );
    }

    #[test]
    fn test_escaping_catch_all_param() {
        static SEGMENTS: &[PathSegment] = &[PathSegment::CatchAllParam("a")];
        assert_eq!(
            PathBuilder::new(SEGMENTS)
                .param("a", "/%20#")
                .build()
                .unwrap(),
            "/%20%23"
        );
    }

    #[test]
    fn test_construction_errors() {
        static SEGMENTS: &[PathSegment] = &[PathSegment::Param("a")];
        assert!(PathBuilder::new(SEGMENTS).build().is_err());
    }

    #[test]
    fn test_clear() {
        static SEGMENTS: &[PathSegment] = &[PathSegment::Param("foo")];
        let path = PathBuilder::new(SEGMENTS)
            .param("foo", 1)
            .query_param("bar", 2)
            .fragment("3");
        assert_eq!(path.build().unwrap(), "1?bar=2#3");
        let path = path
            .cleared_param("nonexistent")
            .cleared_query_param("nonexistent");
        assert_eq!(path.build().unwrap(), "1?bar=2#3");
        let path = path.cleared_fragment().cleared_query_param("bar");
        assert_eq!(path.build().unwrap(), "1");
        let path = path.cleared_param("foo");
        assert!(path.build().is_err());
    }

    #[test]
    fn test_fill() {
        static SEGMENTS: &[PathSegment] = &[PathSegment::Param("foo")];
        let path1 = PathBuilder::new(SEGMENTS)
            .param("foo", 1)
            .query_param("bar", 2)
            .fragment("3");
        let path2 = PathBuilder::new(SEGMENTS).filled_from(&path1);
        assert_eq!(path1.build().unwrap(), "1?bar=2#3");
        assert_eq!(path2.build().unwrap(), "1?bar=2#3");
    }
}
