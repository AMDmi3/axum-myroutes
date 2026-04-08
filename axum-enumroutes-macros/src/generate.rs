// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use proc_macro::TokenStream;
use quote::quote;

use crate::path::PathSegment;
use crate::types::{Enum, Method};

pub fn generate(r#enum: Enum) -> syn::Result<TokenStream> {
    let vis = &r#enum.vis;
    let ident = &r#enum.ident;
    let extractor_ident = quote::format_ident!("Self{}", &r#enum.ident);
    let state_type = &r#enum.state_type;
    let props_type = &r#enum.props_type;

    let mut enum_variants = vec![];
    let mut path_matches = vec![];
    let mut name_matches = vec![];
    let mut props_matches = vec![];
    let mut routes = vec![];
    let mut path_segment_matches = vec![];

    let props_return_type = if r#enum.static_props {
        quote! { &'static #props_type }
    } else {
        quote! { #props_type }
    };

    let props_default_match = if !r#enum.default_props {
        quote! {}
    } else if r#enum.static_props {
        quote! { _ => { static VALUE: #props_type = <#props_type>::default(); &VALUE }, }
    } else {
        quote! { _ => <#props_type>::default(), }
    };

    for variant in &r#enum.variants {
        let variant_ident = &variant.ident;
        let variant_ident_name = variant_ident.to_string();
        let variant_other_attributes = &variant.other_attributes;
        let variant_path = variant.route.path.to_string();
        let variant_handler = &variant.route.handler;
        let method_router_ident = match &variant.route.method {
            Method::Get => quote::format_ident!("get"),
            Method::Post => quote::format_ident!("post"),
        };

        enum_variants.push(quote! {
            #(#variant_other_attributes)*
            #variant_ident,
        });

        path_matches.push(quote! {
            #ident::#variant_ident => #variant_path,
        });

        name_matches.push(quote! {
            #ident::#variant_ident => #variant_ident_name,
        });

        if let Some(prop) = &variant.route.props {
            if r#enum.static_props {
                props_matches.push(quote! {
                    #ident::#variant_ident => { static VALUE: #props_type = #prop; &VALUE },
                });
            } else {
                props_matches.push(quote! {
                    #ident::#variant_ident => { #prop },
                });
            }
        }

        let path_segments: Vec<_> = variant
            .route
            .path
            .segments
            .iter()
            .map(|segment| match segment {
                PathSegment::Static(s) => {
                    quote! {::axum_enumroutes::__private::PathSegment::Static(#s) }
                }
                PathSegment::Param(s) => {
                    quote! {::axum_enumroutes::__private::PathSegment::Param(#s) }
                }
                PathSegment::CatchAllParam(s) => {
                    quote! {::axum_enumroutes::__private::PathSegment::CatchAllParam(#s) }
                }
            })
            .collect();

        path_segment_matches.push(quote! {
            #ident::#variant_ident => { static VALUE: &[::axum_enumroutes::__private::PathSegment] = &[#(#path_segments),*]; VALUE },
        });

        // We need specific layer ordering: for the route extractor to work in the
        // middleware, layer adding route extension must come AFTER all the layer
        // which may use it.
        routes.push(quote! {
            .merge(
                f(
                    ::axum_enumroutes::__private::axum::Router::<#state_type>::new()
                    .route(
                        #ident::#variant_ident.path(),
                        ::axum_enumroutes::__private::axum::routing::#method_router_ident(
                            #variant_handler
                        )
                    )
                )
                .layer(
                    ::axum_enumroutes::__private::axum::Extension(
                        #ident::#variant_ident
                    )
                )
            )
        });
    }

    Ok(quote! {
        #vis enum #ident {
            #(#enum_variants)*
        }

        impl #ident {
            pub fn path(&self) -> &'static str {
                match self {
                    #(#path_matches)*
                }
            }

            pub fn name(&self) -> &'static str {
                match self {
                    #(#name_matches)*
                }
            }

            pub fn props(&self) -> #props_return_type {
                match self {
                    #(#props_matches)*
                    #props_default_match
                }
            }

            pub fn url_for(&self) -> ::axum_enumroutes::PathBuilder {
                ::axum_enumroutes::PathBuilder::new(
                    match self {
                        #(#path_segment_matches)*
                    }
                )
            }

            pub fn to_router_with<F>(f: F) -> ::axum_enumroutes::__private::axum::Router::<#state_type>
                where F: Fn(
                    ::axum_enumroutes::__private::axum::Router::<#state_type>
                ) -> ::axum_enumroutes::__private::axum::Router::<#state_type>
            {
                axum_enumroutes::__private::axum::Router::new()
                #(#routes)*
            }

            pub fn to_router() -> ::axum_enumroutes::__private::axum::Router::<#state_type> {
                Self::to_router_with(std::convert::identity)
            }
        }

        impl<S: ::std::marker::Send + ::std::marker::Sync>
            ::axum_enumroutes::__private::axum::extract::FromRequestParts<S> for #ident
        {
            type Rejection = ::axum_enumroutes::__private::axum::http::StatusCode;

            async fn from_request_parts(
                parts: &mut ::axum_enumroutes::__private::axum::http::request::Parts,
                _: &S,
            ) -> ::std::result::Result<Self, Self::Rejection> {
                parts
                    .extensions
                    .get::<#ident>()
                    .cloned()
                    .ok_or(::axum_enumroutes::__private::axum::http::StatusCode::INTERNAL_SERVER_ERROR)
            }
        }

        #vis struct #extractor_ident {
            route: #ident,
            url_for_self: ::axum_enumroutes::PathBuilder,
        }

        impl #extractor_ident {
            pub fn path(&self) -> &'static str {
                self.route.path()
            }

            pub fn name(&self) -> &'static str {
                self.route.name()
            }

            pub fn props(&self) -> #props_return_type {
                self.route.props()
            }

            pub fn url_for(&self) -> ::axum_enumroutes::PathBuilder {
                self.url_for_self.clone()
            }
        }

        impl<S: ::std::marker::Send + ::std::marker::Sync>
            ::axum_enumroutes::__private::axum::extract::FromRequestParts<S> for #extractor_ident
        {
            type Rejection = ::axum_enumroutes::__private::axum::http::StatusCode;

            async fn from_request_parts(
                parts: &mut ::axum_enumroutes::__private::axum::http::request::Parts,
                state: &S,
            ) -> ::std::result::Result<Self, Self::Rejection> {
                use ::axum_enumroutes::__private::axum::extract::{Path, Query};
                use ::axum_enumroutes::__private::axum::http::StatusCode;

                let route = parts
                    .extensions
                    .get::<#ident>()
                    .cloned()
                    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
                let Path(path_params) = Path::<Vec<(String, String)>>::from_request_parts(parts, state)
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?;
                let Query(query_params) = Query::<Vec<(String, String)>>::from_request_parts(parts, state)
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?;

                let mut url_for_self = route.url_for();
                for (k, v) in path_params {
                    url_for_self = url_for_self.param(k, v);
                }
                for (k, v) in query_params {
                    url_for_self = url_for_self.query_param(k, v);
                }

                Ok(Self {
                    route,
                    url_for_self
                })
            }
        }
    }
    .into())
}
