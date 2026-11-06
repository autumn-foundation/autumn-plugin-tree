# Planning: the tree widget pack

This plan uses three methods: brainstorming, reverse brainstorming and six
thinking hats. The research is in [research.md](research.md).

## Scope

Issue autumn-foundation/autumn#3417 lists the tree view as a niche widget.
The tree view must follow the APG tree pattern, use the shared keyboard core,
and ship as a plugin through the widget plugin seam (autumn#3400).

## Acceptance criteria

| ID | Criterion | Source |
|---|---|---|
| AC1 | The tree view follows the WAI-ARIA APG tree pattern: roles, states and keyboard. | #3417 |
| AC2 | The tree reuses the keyboard core. The pack JS has no arrow-key, Home/End, typeahead or Escape code. | #3417, #3400 |
| AC3 | The widget is a plugin crate outside core. It installs through `WidgetPack`, `PluginAssets` and `Plugin::build`. | #3417, #3400 |
| AC4 | The widget obeys the widget contract: render, passthrough, tokens, class names, CSP and stories. | #3400 |
| AC5 | The pack passes the static conformance checks and the browser gate (axe, console, keyboard). | #3400 |
| AC6 | The widget works with JavaScript off. | #3393 |
| AC7 | A form input binds to a `Changeset<T>` and to `form_for` through `FieldRenderer`. | #3400 |
| AC8 | A dedicated sub-issue exists and links to #3417 (it is #3649). | #3417 |

## Brainstorming

We wrote down all ideas first. We did not judge them.

- An APG tree (`role="tree"`) for file and folder selection.
- A navigation tree for docs sidebars.
- A checkbox tree with tri-state parents.
- A radio tree for one choice.
- Lazy children through htmx.
- "Load more" rows for large levels.
- Virtual scrolling.
- Drag and drop to move nodes.
- Inline rename.
- Search and filter, with match highlight.
- Expand all and collapse all buttons.
- Guide lines and two densities.
- Icons before the label.
- Row actions (menu, delete).
- Server-side cascade and validation of submitted keys.
- `?open=` state in the URL for no-JS users.
- A `treegrid` variant with columns.
- Persist the open state in `localStorage`.

### Selected for version 1

| Idea | Decision | Reason |
|---|---|---|
| APG tree, single and multiple choice | Yes | AC1, AC7 |
| Navigation tree (disclosure, no `role="tree"`) | Yes | The APG, Primer, Carbon and Fluent say: do not use a tree for navigation. |
| Tri-state parents, cascade or independent | Yes | All peers with checkboxes have it. |
| Lazy children through htmx | Yes | Primer and Web Awesome have it. It is the answer for large trees. |
| Server-side resolve of submitted keys | Yes | Security: the server checks each key. |
| Guide lines, compact density | Yes | Small CSS. Common in peers. |
| Icons before the label | Later | The app can put an icon in the label text. |
| Depth and node budgets | Yes | Security: a deep tree must not stop the server. |
| "Load more" rows, search, expand-all buttons | Later | Not in the issue. The app can add buttons. |
| Virtual scrolling, drag and drop, rename, row actions, `treegrid` | No | Outliers. They need much JS. Row actions break the tree pattern. |
| `localStorage` state | No | The server owns state. |

## Reverse brainstorming

Question: "How can we make this tree fail?" Each failure gives a rule.

| How to fail | Rule |
|---|---|
| Send `role="tree"` and the script does not load. The user hears a tree, but the keys do nothing. | The server sends a `<details>` list. The script adds the roles. |
| ArrowDown goes into a closed branch. Focus goes to an item that is not visible. | The keyboard core skips items in a `hidden` subtree. |
| Put a checkbox or link in each tree item. Screen readers read an empty tab stop. | An upgraded item has no focusable child. The native input stays, but it is `hidden`. |
| Let selection follow focus. Each Arrow key changes the form value. | Enter, Space and click select. Focus does not select. |
| Write `style="--depth:3"` for indentation. A strict CSP blocks it. | Indentation comes from nested padding in the pack CSS. |
| Put a label with `<script>` in a node. | maud escapes all labels. |
| Put `javascript:alert(1)` in a node link. | Only relative, `http`, `https` and `mailto` links. Other links become text. |
| Send a tree that is 100,000 levels deep. The renderer overflows the stack. | Stop at 64 levels and 10,000 nodes. |
| Submit a key that is not in the tree. | `TreeField::resolve` drops it. |
| Two trees on one page use the same ids. | Each id starts with the field name. Each key is encoded. |
| Lazy load fails. The user waits for ever. | The loading row shows an error. The next expand tries again. |
| Mark a `hidden` radio `required`. The browser cannot show the error and blocks the submit. | The script moves `required` to `aria-required` on the tree. The server checks it. |
| Copy the arrow-key code into the pack. Two key models drift apart. | The core owns Right, Left and `*` for each `role="tree"`. |

## Six thinking hats

**White (facts).** The APG tree pattern is stable. Support in screen readers
is uneven. GitHub had to declare `aria-level`, `aria-setsize` and
`aria-posinset`, and name each item with `aria-labelledby`. The keyboard core
in `autumn-web` 0.10.0 has `[role=treeitem]` in its default items. It does not skip items in a closed
branch, and it has no Right, Left or `*` for trees. `autumn-web` 0.10.0 on
crates.io has the widget pack seam.

**Red (feelings).** A tree is a strong control for file pickers. It is a bad
control for site menus. Users like a checkbox tree that shows "some" on a
parent.

**Black (risks).**
- The core change is in another repository. Until it ships, Right, Left
  and `*` do nothing in a tree. Mitigation: Up, Down, Home, End, typeahead,
  Enter, Space and click work with `autumn-web` 0.10. The core PR adds the
  other keys and their browser tests.
- `aria-checked="mixed"` on a tree item is an open spec question
  (w3c/aria#1640). Mitigation: a hidden "Partly selected" text is the fallback.
- The upgrade moves DOM nodes. htmx attributes must stay on the `ul`, not on
  `<details>`.

**Yellow (benefits).** The `autumn:init` event fires before roving set-up, so
the upgrade is cheap. The browser submits the native inputs, so the form
needs no script copy of the state. One endpoint serves htmx fragments and
full pages.

**Green (new ideas).** One data model (`TreeNode`) serves both widgets. The
core change (skip hidden subtrees) also helps other nested roving lists.
`TreeField::resolve` gives one server rule for cascade and validation.

**Blue (process).**
1. Red: write failing tests for each behaviour.
2. Green: write the smallest code that passes them.
3. Refactor with all tests green.
4. Review from several angles. Fix the findings.
5. Map each criterion to evidence.

## Design

### Widgets

| Widget | Type | JavaScript |
|---|---|---|
| `nav_tree` | Disclosure navigation: `<nav>`, nested lists, `<details>`, links, `aria-current="page"` | None |
| `TreeField` | `FieldRenderer`. A `<details>` list of radios or checkboxes. The script upgrades it to an APG tree. | `tree.js` |

### Data model

```rust
TreeNode { key: TreeKey, label: String, href: Option<String>, disabled: bool, children: Children }
Children::{ None, Nodes(Vec<TreeNode>), Lazy(String) }
TreeField { nodes, multiple: bool, cascade: Cascade, expanded: BTreeSet<TreeKey>, … }
Cascade::{ Descendants, Independent }
```

### Server markup (JavaScript off)

```html
<ul class="autumn-tree-root" id="f-tree" aria-label="Folders" data-autumn-tree="multiple">
  <li class="autumn-tree-node" id="f-n-src-node" aria-level="1" aria-setsize="1" aria-posinset="1">
    <input type="checkbox" id="f-n-src" name="f" value="src" aria-labelledby="f-n-src-label">
    <details class="autumn-tree-details" open>
      <summary class="autumn-tree-row"><span class="autumn-tree-label" id="f-n-src-label">src</span></summary>
      <ul class="autumn-tree-group" id="f-n-src-group">…</ul>
    </details>
  </li>
</ul>
```

### Upgraded markup (after `autumn:init`)

```html
<ul class="autumn-tree-root autumn-tree--enhanced" role="tree" aria-label="Folders"
    aria-multiselectable="true" data-autumn-roving="…" data-autumn-orientation="vertical"
    data-autumn-typeahead>
  <li class="autumn-tree-node" role="none">
    <input type="checkbox" … hidden>
    <div class="autumn-tree-row" role="treeitem" id="f-n-src-node" tabindex="0"
         aria-expanded="true" aria-owns="f-n-src-group" aria-checked="mixed"
         aria-labelledby="f-n-src-label" aria-level="1" aria-setsize="1" aria-posinset="1">
      …<span class="autumn-tree-label" id="f-n-src-label">src</span>
    </div>
    <ul class="autumn-tree-group" id="f-n-src-group" role="group">…</ul>
  </li>
</ul>
```

The row is the tree item, not the `<li>`. An `<li>` item holds its open
group, so its visible text is the whole branch, and axe reports
`label-content-name-mismatch`. The row holds the label only. `aria-owns`
links the row to its group, as in the APG navigation tree example.

### Who does what

| Part | Owner |
|---|---|
| Up, Down, Home, End, typeahead, roving tab stop | Keyboard core (now) |
| Skip items in a hidden subtree; Right, Left, `*`; `autumn:expand` and `autumn:collapse` events | Keyboard core (autumn PR) |
| Upgrade, click, Enter, Space, Ctrl+A, cascade, tri-state, lazy load | `tree.js` |
| Tri-state at render, cascade on submit, key checks, budgets | Rust |

## Test plan

| Layer | Tests |
|---|---|
| Unit (Rust) | Key encoding, href filter, budgets, tri-state, cascade, `resolve`, markup shape, escaping, passthrough. |
| Static conformance | `run_widget_conformance(&pack())` passes. |
| Browser (Chromium) | Upgrade, no-JS fallback, click, Enter, Space, Ctrl+A, cascade, lazy load, form submit, CSP, axe gate. |
| Core (autumn PR) | Hidden-subtree skip, Right, Left, `*`, RTL, events. |
