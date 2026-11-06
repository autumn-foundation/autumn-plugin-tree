# CLAUDE.md: autumn-plugin-tree

A widget pack for Autumn. One crate: `autumn-plugin-tree`. It has
`TreeField` (an APG tree form field) and `nav_tree` (disclosure navigation).

## Commands

- Format: `cargo fmt --all`
- Lint: `cargo clippy --all-targets --features system-tests -- -D warnings`
- Test: `cargo test`
- Browser tests (Chromium):
  `cargo test --features system-tests --test browser -- --include-ignored`
- Tree-key tests (need autumn `main`):
  `cargo test --features system-tests,core-tree-keys --test browser --config 'patch.crates-io.autumn-web.git="https://github.com/autumn-foundation/autumn"' -- --include-ignored`
- Local autumn checkout: `--config 'patch.crates-io.autumn-web.path="../autumn/autumn"'`

## Layout

| File | Content |
|---|---|
| `src/key.rs` | `TreeKey` and its id token. |
| `src/node.rs` | `TreeNode` and `Children`. |
| `src/href.rs` | `safe_href`, the link filter. |
| `src/budget.rs` | `MAX_DEPTH`, `MAX_NODES`. |
| `src/field.rs` | `TreeField`: render, tri-state, cascade, `resolve`, `branch`. |
| `src/nav.rs` | `nav_tree`. |
| `src/pack.rs` | `pack()`, `ASSETS`, stories, `TreePlugin`. |
| `assets/tree.js` | The upgrade to an APG tree, click, Enter, Space, Ctrl+A, lazy load. |
| `assets/tree.css` | Public tokens only. |
| `docs/` | Plan, research, ADRs. |

## Rules

- **The widget contract of autumn ADR 0030 applies.** Classes are
  `autumn-tree-*`. CSS uses only public tokens. No inline script or style,
  no `style` attribute. Each widget has a story with a `tree-` slug.
  `tests/pack.rs` runs the static conformance check.
- **`tree.js` has no arrow-key, Home/End, typeahead or Escape code.** The
  keyboard core owns them. `tests/pack.rs` fails on `ArrowUp` and others.
- **The server markup has no `role="tree"`.** It must work with JavaScript
  off. The script adds the roles on `autumn:init`.
- **The native inputs are the form state.** The script changes
  `input.checked`, then writes the ARIA state from the inputs.
- **Check all posted keys with `TreeField::resolve`.**
- **Keep the render budgets.** A new walk over the nodes uses `MAX_DEPTH`
  and `MAX_NODES`.
- No `unwrap`, `expect` or `panic!` in library code.
- No behaviour without a test. Markup: `tests/tree_field.rs`,
  `tests/nav_tree.rs`. Server checks: `tests/resolve.rs`. Browser:
  `tests/browser.rs`.
- Comments and docs: short, plain English (ASD-STE100). No review labels in
  comments.
- Record a large decision in `docs/adr/NNNN-*.md`.
