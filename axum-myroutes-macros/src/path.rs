// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

pub enum PathSegment {
    Static(String),
    Param(String),
    CatchAllParam(String),
}

pub struct Path {
    pub segments: Vec<PathSegment>,
}

impl Path {
    pub fn parse(path: &syn::LitStr) -> syn::Result<Path> {
        let mut segments = vec![];
        let mut current = String::new();
        let mut in_param = false;
        let path_str = path.value();
        let mut iter = path_str.chars().peekable();

        loop {
            match iter.next() {
                Some('{') => {
                    if in_param {
                        return Err(syn::Error::new(
                            path.span(),
                            "unexpected `{` within parameter name".to_string(),
                        ));
                    } else if iter.peek() == Some(&'{') {
                        current += "{";
                        iter.next();
                    } else if iter.peek().is_none() {
                        return Err(syn::Error::new(
                            path.span(),
                            "unexpected `{` at end of path".to_string(),
                        ));
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
                            return Err(syn::Error::new(
                                path.span(),
                                "empty parameter name not allowed".to_string(),
                            ));
                        }
                        let param_name = std::mem::take(&mut current);
                        if let Some(param_name) = param_name.strip_prefix('*') {
                            segments.push(PathSegment::CatchAllParam(param_name.into()));
                        } else {
                            segments.push(PathSegment::Param(param_name));
                        }
                        in_param = false;
                    } else if iter.peek() == Some(&'}') {
                        current += "}";
                        iter.next();
                    } else {
                        return Err(syn::Error::new(
                            path.span(),
                            "unexpected dangling `}`".to_string(),
                        ));
                    }
                }
                Some(c) => {
                    current.push(c);
                }
                None => {
                    if in_param {
                        return Err(syn::Error::new(
                            path.span(),
                            "unclosed parameter".to_string(),
                        ));
                    }
                    if !current.is_empty() {
                        segments.push(PathSegment::Static(current));
                    }
                    return Ok(Self { segments });
                }
            }
        }
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in &self.segments {
            match segment {
                PathSegment::Static(text) => {
                    for c in text.chars() {
                        match c {
                            '{' | '}' => write!(f, "{c}{c}")?,
                            _ => write!(f, "{c}")?,
                        }
                    }
                }
                PathSegment::Param(name) => {
                    write!(f, "{{{name}}}")?;
                }
                PathSegment::CatchAllParam(name) => {
                    write!(f, "{{*{name}}}")?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dump() {
        let samples = ["/", "{foo}", "/{foo}{bar}", "{{}}", "{{{bar}}}", "{*rest}"];
        for sample in &samples {
            let literal: syn::LitStr = syn::parse_quote!(#sample);
            let path = Path::parse(&literal).unwrap();
            assert_eq!(path.to_string(), sample.to_string());
        }
    }

    #[test]
    fn test_parse_errors() {
        let samples = ["{", "}", "{abc", "abc}", "{}", "{foo{}"];
        for sample in &samples {
            let literal: syn::LitStr = syn::parse_quote!(#sample);
            assert!(Path::parse(&literal).is_err());
        }
    }
}
