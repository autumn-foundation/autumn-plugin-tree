# ADR 0001: A disclosure list that the script upgrades to a tree

- Status: Accepted
- Date: 2026-11-06

## Context

The APG tree pattern needs JavaScript for its keys. A `role="tree"` with no
keys is worse than a plain list. Autumn widgets must work with JavaScript
off. Screen reader support for trees is uneven. GitHub had to declare
`aria-level`, `aria-setsize` and `aria-posinset`, and name each item with
`aria-labelledby`.

## Options

1. The server sends `role="tree"`. The script adds the keys.
2. The server sends a nested `<details>` list of native inputs. The script
   upgrades it to `role="tree"` on `autumn:init`.
3. No `role="tree"`. A disclosure list only.

## Decision

Option 2 for `TreeField`. Option 3 for `nav_tree`.

- The server sends native radios or checkboxes. The browser posts them.
- The server declares level, position and set size on each item.
- The script moves the label into a row, sets the roles, hides the inputs
  and keeps them in the form.
- The row is the `treeitem`. The `<li>` is `role="none"`. The row names its
  group in `aria-owns`. An `<li>` item would hold its open group, so its
  visible text would not match its name (axe
  `label-content-name-mismatch`).
- The keyboard core owns the arrow keys, Home, End and typeahead. The pack
  owns click, Enter, Space and Ctrl+A.
- Multiple trees use `aria-checked`, with `mixed` for a parent with some
  children checked. A hidden "partly selected" text is the fallback,
  because `mixed` on a tree item is an open spec question (w3c/aria#1640).
- Navigation uses `nav_tree`. The APG says the disclosure pattern is better
  for web sites.

## Consequences

- Without JavaScript, the tree is a native form. It has no tree keys.
- One renderer serves the page and the lazy fragments.
- The keyboard core needs tree keys: skip hidden subtrees, Right, Left and
  `*` (autumn-foundation/autumn#3417). Until a release has them, the
  chevron and the other keys still work.
