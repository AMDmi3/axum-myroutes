// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::str::FromStr;

pub enum Method {
    Get,
    Post,
}

pub struct ParseMethodError;

impl FromStr for Method {
    type Err = ParseMethodError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "get" => Ok(Self::Get),
            "post" => Ok(Self::Post),
            _ => Err(ParseMethodError),
        }
    }
}
