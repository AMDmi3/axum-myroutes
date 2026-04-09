// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::str::FromStr;

// Each variant corresponds to axum funtion from method_routing
// See https://docs.rs/axum/latest/axum/routing/method_routing/index.html
pub enum Method {
    Any,
    Connect,
    Delete,
    Get,
    Head,
    Options,
    Patch,
    Post,
    Put,
    Trace,
}

pub struct ParseMethodError;

impl FromStr for Method {
    type Err = ParseMethodError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "any" => Ok(Self::Any),
            "connect" => Ok(Self::Connect),
            "delete" => Ok(Self::Delete),
            "get" => Ok(Self::Get),
            "head" => Ok(Self::Head),
            "options" => Ok(Self::Options),
            "patch" => Ok(Self::Patch),
            "post" => Ok(Self::Post),
            "put" => Ok(Self::Put),
            "trace" => Ok(Self::Trace),
            _ => Err(ParseMethodError),
        }
    }
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Connect => "connect",
            Self::Delete => "delete",
            Self::Get => "get",
            Self::Head => "head",
            Self::Options => "options",
            Self::Patch => "patch",
            Self::Post => "post",
            Self::Put => "put",
            Self::Trace => "trace",
        }
    }
}
