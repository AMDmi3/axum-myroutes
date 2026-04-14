# Changelog

## unreleased

- **added**: `path_param()`, `cleared_path_param()` methods for `PathBuilder`.
  These provide stricter API and return error when nonexistent path parameter
  is provided.
- **breaking**: `PathBuilder::param()` now falls back to setting query parameter
  if there's no path parameter with the provided name.
- **breaking**: `PathBuilder::clear_param()` now clears both query and path parameters.
- **breaking**: `PathBuilder::filled_from()` is now fallible as it checks paths
  compatibility: parameter sets used in both paths must now match (but order is irrelevant).
  This restriction may be relaxed in the future.
- **breaking**: `PathConstructionError` was renamed to `PathBuilderError` and is now
  `non_exhaustive`.
- **added**: `cleared_`/`retained_` methods for path/query/all params and the fragment.

## 0.1.2

- **added**: `base()` method for route extractor.

## 0.1.1

- **added**: `clear_{param,query_param,fragment}` methods for `PathBuilder`.
- **added**: `filled_from` method for `PathBuilder`.
- **added**: `url_for_self` method for current route extractor, returning
  a reference to `PathBuilder`, allowing read-only access to the current route
  to consreuct an unmodified path to self, or copy params to another `PathBuilder`.

## 0.1.0

- Initial release.
