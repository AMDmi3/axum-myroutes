// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use proc_macro::TokenStream;
use quote::quote;

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
        let variant_path = &variant.route.path;
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

        routes.push(quote! {
            .route(
                #ident::#variant_ident.path(),
                ::axum_enumroutes::__private::axum::routing::#method_router_ident(
                    #variant_handler
                )
                .layer(
                    // This adds the route as an extension to both request and response
                    // The former can be used by the extractor, while the latter can be used by
                    // the middleware, which won't have access to request extension due to layer
                    // ordering
                    ::axum_enumroutes::__private::axum::middleware::from_fn(
                        async |mut request: ::axum_enumroutes::__private::axum::extract::Request,
                               next: ::axum_enumroutes::__private::axum::middleware::Next| {
                            request.extensions_mut().insert(#ident::#variant_ident);
                            let mut response = next.run(request).await;
                            response.extensions_mut().insert(#ident::#variant_ident);
                            response
                        }
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
                ::axum_enumroutes::PathPattern::new(self.path()).url_for()
            }

            // We could make this one generic on S: Clone + Send + Sync + 'static
            // and take/return axum::Router::<S>, however that won't work because
            // - given handler(_: axum::extract::State<MyAppState>)
            // - get(handler) would produce MethodRouter<MyAppState>
            // - axum::Router<S> won't accept it unless S is bounded to MyAppState
            // - but we can't get MyAppState from handler
            // Maybe adding assoc. type to axum::Handler would help, experiments needed
            // for now, solely because of this problem, we require state type to be
            // specified in #[route]
            pub fn add_to_router(
                router: ::axum_enumroutes::__private::axum::Router::<#state_type>
            ) -> ::axum_enumroutes::__private::axum::Router::<#state_type>
            {
                router
                #(#routes)*
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
