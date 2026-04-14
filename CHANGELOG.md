# Changelog

## unreleased

- **added**: `base()` method for route extractor.

## 0.1.1

- **added**: `clear_{param,query_param,fragment}` methods for `PathBuilder`.
- **added**: `filled_from` method for `PathBuilder`.
- **added**: `url_for_self` method for current route extractor, returning
  a reference to `PathBuilder`, allowing read-only access to the current route
  to consreuct an unmodified path to self, or copy params to another `PathBuilder`.

## 0.1.0

- Initial release.
