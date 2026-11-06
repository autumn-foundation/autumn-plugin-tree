# Changelog

## [Unreleased]

### Added

- `TreeField`: a tree of radios or checkboxes, bound to a form field. The
  server sends a native `<details>` list. `tree.js` upgrades it to a
  WAI-ARIA APG tree.
- Tri-state parents, `Cascade::Descendants` and `Cascade::Independent`.
- `TreeField::resolve`: the server check of the posted keys.
- Lazy children through htmx, with `TreeField::branch` for the response.
- `nav_tree`: navigation links in a disclosure list.
- `TreePlugin` and `pack()`: assets, stories and conformance.
- Render budgets: `MAX_DEPTH` and `MAX_NODES`.
