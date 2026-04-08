// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This crate provides convenient and reliable way to work with routes
//! in `axum`. Define routes along with their methods, paths and handlers
//! in an enum, add these to `axum::Router` with a single call, then refer
//! to any route (and construct a link to it) via corresponding enum variant,
//! which provides compile time checked internal links for your application.
//! Also, axum extractor is provided for handlers to be aware of their routes,
//! and capable of constructing links to themselves.
//!
//! # Example
//!
//! ```no_run
//! use axum_enumroutes::routes;
//!
//! // Specify routes
//! #[derive(Clone, Copy)]
//! #[routes]
//! enum Route {
//!     #[get("/items.{id}", handler = item_by_id)]
//!     ItemById,
//! }
//!
//! # fn test() {
//! // Inspect routes and construct paths
//! assert_eq!(Route::ItemById.path(), "/items/{id}");
//! assert_eq!(Route::ItemById.name(), "ItemById");
//! // Parameter needs to be specified
//! assert!(Route::ItemById.url_for().build().is_err());
//! assert_eq!(Route::ItemById.url_for().param("id", 1).build().unwrap(), "/items/1".to_string());
//! # }
//!
//! // `Self``Route` is an extractor generated for `Route` enum
//! async fn item_by_id(route: SelfRoute) {
//!     assert_eq!(route.path(), "/items/{id}");
//!     assert_eq!(route.name(), "ItemById");
//!     // Current parameters are already filled...
//!     assert!(route.url_for().build().is_ok());
//!     // ...but may be overwritten
//!     assert_eq!(route.url_for().param("id", 2).build().unwrap(), "/items/2".to_string());
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     // Add all routes from enum to an axum::Router
//!     let app = Route::to_router();
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```
//!
//! ## Router with state
//!
//! If router with state is used (e.g. `.with_state()` is called on a router), the
//! state type must be passed to `#[routes]` argument:
//!
//! ```no_run
//! use axum_enumroutes::routes;
//!
//! #[derive(Clone)]
//! struct AppState;
//!
//! async fn handler(_: axum::extract::State<AppState>) {}
//!
//! #[derive(Clone, Copy)]
//! #[routes(state_type = AppState)]
//! enum Route {
//!     #[get("/", handler = handler)]
//!     Home,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     // Add all routes from enum to an axum::Router
//!     let app = Route::to_router().with_state(AppState{});
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
//! # use axum_enumroutes::routes;
//! # async fn _handler(){}
//! #[derive(Clone, Copy)]
//! #[routes]
//! enum Route {
//!     #[get("/{id}", handler = _handler)]
//!     ItemById,
//! }
//!
//! assert_eq!(Route::ItemById.path(), "/{id}");
//! assert_eq!(Route::ItemById.name(), "ItemById");
//!
//! assert_eq!(
//!     Route::ItemById.url_for().param("id", 123).build().unwrap(),
//!     "/123".to_string()
//! );
//! assert!(
//!     // required parameter id is missing
//!     Route::ItemById.url_for().build().is_err()
//! );
//! assert_eq!(
//!     Route::ItemById
//!         .url_for()
//!         .param("id", 123)
//!         .query_param("foo", "bar")
//!         .fragment("frag")
//!         .build()
//!         .unwrap(),
//!     "/123?foo=bar#frag".to_string()
//! );
//! ```
//!
//! ## Route props
//!
//! A type to store additional route properties can be provided, set per-route, and
//! retrieved with route method:
//!
//! ```
//! # use axum_enumroutes::routes;
//! # async fn _handler(){}
//! #[derive(Default)]
//! struct RouteProps {
//!     requires_auth: bool,
//! }
//!
//! #[derive(Clone, Copy)]
//! #[routes(props_type = RouteProps)]
//! enum Route {
//!     #[get("/", handler = _handler)]
//!     Public,
//!     #[get("/private", handler = _handler, props = RouteProps { requires_auth: true, ..Default::default() })]
//!     Private,
//! }
//!
//! assert_eq!(Route::Public.props().requires_auth, false);
//! assert_eq!(Route::Private.props().requires_auth, true);
//! ```
//!
//! There are options to toggle `Default` requirement on props type, and to allow
//! static construction of props, enabled through [`routes`] arguments.
//!
//! ## Extractors
//!
//! ```
//! # use axum_enumroutes::routes;
//! // SelfRoute extractor provided for Route enum
//! async fn item_by_id(route: SelfRoute) {
//!     // Same methods as route variant
//!     assert_eq!(route.path(), "/");
//!     assert_eq!(route.name(), "Home");
//!
//!     // In extractor, parameters are already filled,
//!     // so path to self can be constructed right away
//!     assert!(route.url_for().build().is_ok());
//! }
//!
//! #[derive(Clone, Copy)]
//! #[routes]
//! enum Route {
//!     #[get("/{id}", handler = item_by_id)]
//!     ItemById,
//! }
//! ```

#![forbid(unsafe_code)]

use indexmap::IndexMap;
use std::collections::HashMap;

/// Main attribute macro for routes enum.
///
/// Use this on routes enum. Each enum variant should also be marked
/// with an attribute defining a HTTP method and path to handle, and
/// the handler.
///
/// # Example
///
/// ```
/// use axum_enumroutes::routes;
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
///   you may want to derive `Copy` as well).
/// - Variants must not contain any data fields or a discriminant,
///   each variant must have a single route attribute.
///
/// # Macro attribute arguments
///
/// - `state_type` (default is unit type (`()`)) - type for route state. If you
///   call `.with_state()` on the router, it should be the same type.
/// - `props_type` (default is unit type (`()`)) - type for route properties.
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
/// Allowed attributes corresponding to HTTP methods: `get`, `post`.
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
/// - `url_for() -> PathBuilder` - return a path constructor for this route.
/// - `to_router() -> axum::Router` - create axum router with specified routes.
/// - `to_router_with(f: Fn(axum::Router) -> axum::Router) -> axum::Router` - create
///   axum router with custom modifications (such as adding middleware). This is required
///   to ensure layer ordering, as layers which use route extractor must be registered
///   before a layer which adds corresponding extension. In other words, to add middleware
///   with access to route extractor, you must add it through this method, and not on
///   the already constructed router.
///
/// Additionally, extractor type is generated, named with `Self` prefix (e.g.
/// `SelfRoute` for `enum Route`), with the same methods except for `to_router*`.
/// Unlike the route enum variant, `url_for()` for this type returns path constructor
/// with parameters already filled from the current request, so you can construct URL
/// to self from it right away, or override some parameters if necessary.
pub use axum_enumroutes_macros::routes;

#[doc(hidden)]
pub mod __private {
    pub use axum;
}

#[derive(Clone)]
enum PathSegment {
    Static(String),
    Param(String),
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum PathParsingError {
    #[error("empty parameter name")]
    EmptyParamName,
    #[error("invalid path syntax")]
    InvalidPathSyntax,
}

#[derive(thiserror::Error, Debug)]
pub enum PathConstructionError {
    #[error("missing path parameter {0}")]
    MissingPathParam(String),
    #[error("cannot parse route path")]
    PathParsingError(#[from] PathParsingError),
}

#[doc(hidden)]
#[derive(Clone)]
pub struct PathPattern {
    segments: Vec<PathSegment>,
    deferred_parsing_error: Option<PathParsingError>,
}

impl PathPattern {
    pub fn new(path: &str) -> Self {
        match Self::try_new(path) {
            Ok(s) => s,
            Err(e) => PathPattern {
                segments: vec![],
                deferred_parsing_error: Some(e),
            },
        }
    }

    pub fn try_new(path: &str) -> Result<Self, PathParsingError> {
        let mut segments = vec![];
        let mut current = String::new();
        let mut in_param = false;
        let mut iter = path.chars().peekable();

        loop {
            match iter.next() {
                Some('{') => {
                    if in_param {
                        return Err(PathParsingError::InvalidPathSyntax);
                    } else if iter.peek() == Some(&'{') {
                        current += "{";
                        iter.next();
                    } else if iter.peek().is_none() {
                        return Err(PathParsingError::InvalidPathSyntax);
                    } else {
                        if !current.is_empty() {
                            segments.push(PathSegment::Static(std::mem::take(&mut current)));
                        }
                        in_param = true;
                    }
                }
                Some('}') => {
                    if in_param {
                        if current.is_empty() {
                            return Err(PathParsingError::EmptyParamName);
                        }
                        segments.push(PathSegment::Param(std::mem::take(&mut current)));
                        in_param = false;
                    } else if iter.peek() == Some(&'}') {
                        current += "}";
                        iter.next();
                    } else {
                        return Err(PathParsingError::InvalidPathSyntax);
                    }
                }
                Some(c) => {
                    current.push(c);
                }
                None => {
                    if !current.is_empty() {
                        segments.push(PathSegment::Static(current));
                    }
                    return Ok(Self {
                        segments,
                        deferred_parsing_error: None,
                    });
                }
            }
        }
    }

    pub fn url_for(self) -> PathBuilder {
        PathBuilder::new(self)
    }
}

/// Route path builder.
///
/// This allows to fill a route path pattern with given parameter
/// values and construct a complete path.
#[derive(Clone)]
pub struct PathBuilder {
    path_pattern: PathPattern,
    path_params: HashMap<String, String>,
    query_params: IndexMap<String, String>,
    fragment: Option<String>,
}

impl PathBuilder {
    #[doc(hidden)]
    pub fn new(path_pattern: PathPattern) -> Self {
        Self {
            path_pattern,
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

    /// Adds or updates query parameter.
    pub fn query_param<K, V>(mut self, k: K, v: V) -> Self
    where
        K: Into<String>,
        V: std::fmt::Display,
    {
        self.query_params.insert(k.into(), format!("{}", v));
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
        if let Some(error) = &self.path_pattern.deferred_parsing_error {
            return Err(error.clone().into());
        };

        let mut res = String::new();
        for segment in &self.path_pattern.segments {
            match segment {
                PathSegment::Static(s) => {
                    res += s;
                }
                PathSegment::Param(key) => {
                    if let Some(key) = key.strip_prefix('*')
                        && let Some(value) = self.path_params.get(key)
                    {
                        res += &url_escape::encode_path(&value);
                    } else if let Some(value) = self.path_params.get(key) {
                        res += &url_escape::encode_component(&value);
                    } else {
                        return Err(PathConstructionError::MissingPathParam(key.clone()));
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
    use super::*;

    #[test]
    fn test_basic_parse_build() {
        let path = PathPattern::new("/static/{{}}/a{foo}/{bar}b/c{baz}c")
            .url_for()
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
    fn test_parsing() -> Result<(), anyhow::Error> {
        assert_eq!(
            PathPattern::try_new("").unwrap().url_for().build().unwrap(),
            ""
        );
        assert_eq!(PathPattern::try_new("{{")?.url_for().build()?, "{");
        assert_eq!(PathPattern::try_new("}}")?.url_for().build()?, "}");
        assert_eq!(
            PathPattern::try_new("{a}")?
                .url_for()
                .param("a", "1")
                .build()?,
            "1"
        );
        assert_eq!(
            PathPattern::try_new("{a}{b}")?
                .url_for()
                .param("a", "1")
                .param("b", "2")
                .build()?,
            "12"
        );
        Ok(())
    }

    #[test]
    fn test_parsing_errors() {
        assert!(PathPattern::try_new("/{}").is_err());
        assert!(PathPattern::try_new("/{").is_err());
        assert!(PathPattern::try_new("/}").is_err());
        assert!(PathPattern::try_new("/{foo{}}").is_err());
        assert!(PathPattern::try_new("/{foo{{}}}").is_err());
        assert!(PathPattern::try_new("/{foo}}").is_err());
        assert!(PathPattern::try_new("/{{foo}").is_err());
    }

    #[test]
    fn test_escaping() -> Result<(), anyhow::Error> {
        assert_eq!(
            PathPattern::new("{a}")
                .url_for()
                .param("a", "/%20#")
                .build()?,
            "%2F%2520%23"
        );
        assert_eq!(
            PathPattern::new("{*a}")
                .url_for()
                .param("a", "/%20#")
                .build()?,
            "/%20%23"
        );
        Ok(())
    }

    #[test]
    fn test_construction_errors() -> Result<(), anyhow::Error> {
        assert!(PathPattern::try_new("{a}")?.url_for().build().is_err());
        Ok(())
    }
}
